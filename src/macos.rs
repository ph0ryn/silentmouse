use crate::error::SilentMouseError;
use crate::types::{ClickResult, Rect, WindowPoint, WindowTarget};
use core_foundation::array::CFArray;
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::geometry as legacy_geometry;
use core_graphics::geometry::CGRect;
use core_graphics::window::{
    CGWindowID, create_description_from_array, kCGWindowBounds, kCGWindowIsOnscreen,
    kCGWindowNumber, kCGWindowOwnerPID,
};
use libc::{RTLD_NOW, c_void, dlopen, dlsym, pid_t};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSEventType};
use objc2_core_foundation::CGPoint;
use objc2_core_graphics::{CGEvent, CGEventField, CGEventFlags};
use std::ffi::CString;
use std::mem;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type SetWindowLocationFn = unsafe extern "C" fn(*const CGEvent, CGPoint);

const CG_EVENT_FIELD_MOUSE_SUBTYPE: u32 = 7;
const MOUSE_SUBTYPE: i64 = 3;
const NSEVENT_MOUSE_MOVED: usize = 5;
const NSEVENT_LEFT_MOUSE_DOWN: usize = 1;
const NSEVENT_LEFT_MOUSE_UP: usize = 2;
const NSEVENT_LEFT_MOUSE_DRAGGED: usize = 6;

#[derive(Clone, Copy, Debug)]
pub enum MouseEventKind {
    Move,
    Down,
    Drag,
    Up,
}

pub struct MouseEventResult {
    pub window_id: u32,
    pub pid: i32,
    pub event_name: &'static str,
    pub target_was_active: bool,
    pub used_background_flag: bool,
}

pub fn click_window(
    window_id: u32,
    point: WindowPoint,
    duration: Duration,
) -> Result<ClickResult, SilentMouseError> {
    ensure_accessibility_prompted()?;
    let target = describe_window(window_id)?;
    let set_window_location = resolve_set_window_location()?;
    post_left_click(&target, point, duration, set_window_location)?;

    Ok(ClickResult {
        window_id: target.window_id,
        pid: target.pid,
        target_was_active: target.is_active,
        used_background_flag: !target.is_active,
    })
}

pub fn post_mouse_event(
    window_id: u32,
    point: WindowPoint,
    kind: MouseEventKind,
) -> Result<MouseEventResult, SilentMouseError> {
    ensure_accessibility_prompted()?;
    let target = describe_window(window_id)?;
    let set_window_location = resolve_set_window_location()?;
    let event = make_mouse_event(kind, &target, point, 1, set_window_location)?;
    CGEvent::post_to_pid(target.pid as pid_t, Some(&event));

    Ok(MouseEventResult {
        window_id: target.window_id,
        pid: target.pid,
        event_name: kind.name(),
        target_was_active: target.is_active,
        used_background_flag: !target.is_active,
    })
}

fn describe_window(window_id: u32) -> Result<WindowTarget, SilentMouseError> {
    let window_array = CFArray::from_copyable(&[window_id as CGWindowID]);
    let descriptions = create_description_from_array(window_array)
        .filter(|array| !array.is_empty())
        .ok_or(SilentMouseError::WindowNotFound(window_id))?;
    let description = descriptions
        .iter()
        .next()
        .ok_or(SilentMouseError::WindowNotFound(window_id))?;

    let actual_window_id = dict_u32(&description, unsafe { kCGWindowNumber }).ok_or(
        SilentMouseError::MissingWindowField {
            window_id,
            field: "kCGWindowNumber",
        },
    )?;
    if actual_window_id != window_id {
        return Err(SilentMouseError::WindowNotFound(window_id));
    }

    let is_onscreen = dict_bool(&description, unsafe { kCGWindowIsOnscreen }).unwrap_or(false);
    if !is_onscreen {
        return Err(SilentMouseError::WindowOffscreen(window_id));
    }

    let pid = dict_i32(&description, unsafe { kCGWindowOwnerPID }).ok_or(
        SilentMouseError::MissingWindowField {
            window_id,
            field: "kCGWindowOwnerPID",
        },
    )?;
    let bounds = dict_rect(&description, unsafe { kCGWindowBounds }).ok_or(
        SilentMouseError::MissingWindowField {
            window_id,
            field: "kCGWindowBounds",
        },
    )?;
    let is_active = is_pid_active(pid)?;

    Ok(WindowTarget {
        window_id,
        pid,
        bounds,
        is_active,
    })
}

fn post_left_click(
    target: &WindowTarget,
    point: WindowPoint,
    duration: Duration,
    set_window_location: SetWindowLocationFn,
) -> Result<(), SilentMouseError> {
    let down = make_mouse_event(MouseEventKind::Down, target, point, 91, set_window_location)?;
    let up = make_mouse_event(MouseEventKind::Up, target, point, 92, set_window_location)?;

    CGEvent::post_to_pid(target.pid as pid_t, Some(&down));
    thread::sleep(duration);
    CGEvent::post_to_pid(target.pid as pid_t, Some(&up));
    Ok(())
}

fn make_mouse_event(
    kind: MouseEventKind,
    target: &WindowTarget,
    point: WindowPoint,
    event_number: i64,
    set_window_location: SetWindowLocationFn,
) -> Result<Retained<CGEvent>, SilentMouseError> {
    let absolute_point = target.bounds.screen_point(point);
    let screen_point = CGPoint::new(absolute_point.x, absolute_point.y);
    let event = create_nsevent_cgevent(
        kind.nsevent_type(),
        screen_point,
        target.window_id,
        event_number,
    )?;

    if !target.is_active {
        CGEvent::set_flags(Some(&event), CGEventFlags::MaskCommand);
    }
    CGEvent::set_location(Some(&event), screen_point);
    CGEvent::set_integer_value_field(Some(&event), CGEventField(3), 0);
    CGEvent::set_integer_value_field(
        Some(&event),
        CGEventField(CG_EVENT_FIELD_MOUSE_SUBTYPE),
        MOUSE_SUBTYPE,
    );
    CGEvent::set_integer_value_field(Some(&event), CGEventField(91), i64::from(target.window_id));
    CGEvent::set_integer_value_field(Some(&event), CGEventField(92), i64::from(target.window_id));
    CGEvent::set_timestamp(Some(&event), current_timestamp_nanos());

    unsafe {
        set_window_location(&*event as *const CGEvent, CGPoint::new(point.x, point.y));
    }

    Ok(event)
}

impl MouseEventKind {
    fn nsevent_type(self) -> usize {
        match self {
            Self::Move => NSEVENT_MOUSE_MOVED,
            Self::Down => NSEVENT_LEFT_MOUSE_DOWN,
            Self::Drag => NSEVENT_LEFT_MOUSE_DRAGGED,
            Self::Up => NSEVENT_LEFT_MOUSE_UP,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Down => "down",
            Self::Drag => "drag",
            Self::Up => "up",
        }
    }
}

fn create_nsevent_cgevent(
    event_type: usize,
    location: CGPoint,
    window_id: u32,
    event_number: i64,
) -> Result<Retained<CGEvent>, SilentMouseError> {
    let ns_event = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
        NSEventType(event_type),
        location,
        NSEventModifierFlags::empty(),
        0.0,
        window_id as isize,
        None,
        event_number as isize,
        1,
        1.0,
    )
    .ok_or(SilentMouseError::EventCreationFailed)?;

    ns_event
        .CGEvent()
        .ok_or(SilentMouseError::EventCreationFailed)
}

fn dict_i32(
    dict: &CFDictionary<CFString, CFType>,
    key: core_foundation::string::CFStringRef,
) -> Option<i32> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };
    let value = dict.find(&key)?;
    let number = unsafe { CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as _) };
    number.to_i32()
}

fn dict_u32(
    dict: &CFDictionary<CFString, CFType>,
    key: core_foundation::string::CFStringRef,
) -> Option<u32> {
    dict_i32(dict, key).and_then(|value| u32::try_from(value).ok())
}

fn dict_bool(
    dict: &CFDictionary<CFString, CFType>,
    key: core_foundation::string::CFStringRef,
) -> Option<bool> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };
    let value = dict.find(&key)?;
    let boolean = unsafe { CFBoolean::wrap_under_get_rule(value.as_CFTypeRef() as _) };
    Some(boolean.into())
}

fn dict_rect(
    dict: &CFDictionary<CFString, CFType>,
    key: core_foundation::string::CFStringRef,
) -> Option<Rect> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };
    let value = dict.find(&key)?;
    let mut rect = CGRect::new(
        &legacy_geometry::CGPoint::new(0.0, 0.0),
        &legacy_geometry::CGSize::new(0.0, 0.0),
    );
    let ok =
        unsafe { CGRectMakeWithDictionaryRepresentation(value.as_CFTypeRef() as _, &mut rect) };
    if !ok {
        return None;
    }

    Some(Rect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    })
}

fn resolve_set_window_location() -> Result<SetWindowLocationFn, SilentMouseError> {
    let path = CString::new("/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics")
        .expect("CoreGraphics path has no interior nul");
    let symbol_name = CString::new("CGEventSetWindowLocation").expect("symbol has no interior nul");
    let handle = unsafe { dlopen(path.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        return Err(SilentMouseError::WindowLocationSetterUnavailable);
    }

    let symbol = unsafe { dlsym(handle, symbol_name.as_ptr()) };
    if symbol.is_null() {
        return Err(SilentMouseError::WindowLocationSetterUnavailable);
    }

    Ok(unsafe { mem::transmute::<*mut c_void, SetWindowLocationFn>(symbol) })
}

fn is_pid_active(pid: i32) -> Result<bool, SilentMouseError> {
    let class = class!(NSRunningApplication);
    let app: *mut AnyObject =
        unsafe { msg_send![class, runningApplicationWithProcessIdentifier: pid as pid_t] };
    if app.is_null() {
        return Err(SilentMouseError::ActiveStateUnavailable(pid));
    }

    Ok(unsafe { msg_send![app, isActive] })
}

fn current_timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn ensure_accessibility_prompted() -> Result<(), SilentMouseError> {
    if unsafe { AXIsProcessTrusted() } {
        return Ok(());
    }

    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let prompt = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key, prompt)]);

    if unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) } {
        Ok(())
    } else {
        Err(SilentMouseError::AccessibilityPermissionRequired)
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGRectMakeWithDictionaryRepresentation(
        dict: core_foundation::dictionary::CFDictionaryRef,
        rect: *mut CGRect,
    ) -> bool;
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef)
    -> bool;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
