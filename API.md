# Playwright Python API vs Sparkle Rust API

Generated: 2026-01-21

This document compares Sparkle's current Rust implementation against the official
Playwright Python API. Status values:

- Implemented: feature exists and is wired up
- Stub only: type/option exists but behavior is not implemented
- Missing: not present in Sparkle

## Constraints

Sparkle uses WebDriver via `thirtyfour`. Some Playwright features depend on the
Playwright protocol or CDP, so they are either missing or require alternative
implementations (network interception, tracing, video, etc.).

## Top-Level API Coverage

### Playwright

| Feature | Status | Notes |
| --- | --- | --- |
| chromium | Implemented | Returns BrowserType |
| firefox | Implemented | Launch returns NotImplemented |
| webkit | Implemented | Launch returns NotImplemented |
| stop() | Implemented | No-op cleanup |
| devices | Missing | Device descriptors |
| request | Missing | APIRequest |
| selectors | Missing | Custom selector engines |

### BrowserType

| Feature | Status | Notes |
| --- | --- | --- |
| launch() | Implemented | Chromium only |
| launch_persistent_context() | Missing | Persistent profile |
| connect() | Implemented | Remote WebDriver connection (Chromium only) |
| connect_over_cdp() | Implemented | CDP connection via WebDriver (Chromium only) |
| executable_path() | Implemented | Chromium only |
| name | Implemented | BrowserName |

Launch options coverage:

| Option | Status | Notes |
| --- | --- | --- |
| headless | Implemented | Defaults to true |
| args | Implemented | Applied to Chromium caps |
| executable_path | Stub only | Defined but unused in launch |
| slow_mo | Stub only | Defined but unused |
| timeout | Stub only | Defined but unused |
| downloads_path | Stub only | Defined but unused |
| devtools | Stub only | Defined but unused |
| channel | Stub only | Defined but unused |
| chromium_sandbox | Stub only | Defined but unused |
| env | Stub only | Defined but unused |
| proxy | Stub only | Defined but unused |
| traces_dir | Stub only | Defined but unused |
| handle_sighup/sigint/sigterm | Stub only | Defined but unused |

Connect options coverage:

| Option | Status | Notes |
| --- | --- | --- |
| timeout | Implemented | Connection timeout with retry |
| slow_mo | Implemented | Stored but not yet enforced |
| headers | Implemented | Custom WebDriver headers |
| args | Implemented | Browser arguments |
| executable_path | Implemented | Browser binary path |
| env | Implemented | Environment variables |
| channel | Stub only | Not yet supported |

ConnectOverCdp options coverage:

| Option | Status | Notes |
| --- | --- | --- |
| timeout | Implemented | Connection timeout with retry |
| slow_mo | Implemented | Stored but not yet enforced |
| headers | Implemented | Custom headers |

## Browser Coverage

| Feature | Status | Notes |
| --- | --- | --- |
| new_context() | Implemented | Uses BrowserContextOptions |
| new_page() | Implemented | Creates context + page |
| contexts() | Implemented | Returns tracked contexts |
| close() | Implemented | Closes contexts + adapter |
| is_connected() | Partial | `is_closed()` exists |
| version() | Implemented | via adapter |
| browser_type | Missing | No accessor |
| new_browser_cdp_session() | Missing | CDP |

## BrowserContext Coverage

| Feature | Status | Notes |
| --- | --- | --- |
| new_page() | Implemented | Creates page |
| pages() | Implemented | Returns pages |
| close() | Implemented | Closes pages |
| browser() | Missing | No parent accessor |
| set_default_timeout() | Missing | Default timeout mgmt |
| set_default_navigation_timeout() | Missing | Default nav timeout |

Context option coverage:

| Option | Status | Notes |
| --- | --- | --- |
| accept_downloads | Stub only | Defined but unused |
| bypass_csp | Stub only | Defined but unused |
| color_scheme | Stub only | Defined but unused |
| device_scale_factor | Stub only | Defined but unused |
| extra_http_headers | Stub only | Defined but unused |
| geolocation | Stub only | Defined but unused |
| has_touch | Stub only | Defined but unused |
| http_credentials | Stub only | Defined but unused |
| ignore_https_errors | Stub only | Defined but unused |
| is_mobile | Stub only | Defined but unused |
| java_script_enabled | Stub only | Defined but unused |
| locale | Stub only | Defined but unused |
| offline | Stub only | Defined but unused |
| permissions | Stub only | Defined but unused |
| proxy | Stub only | Defined but unused |
| user_agent | Stub only | Defined but unused |
| viewport | Stub only | Defined but unused |
| timezone_id | Stub only | Defined but unused |
| base_url | Stub only | Defined but unused |
| strict_selectors | Stub only | Defined but unused |
| service_workers | Stub only | Defined but unused |
| record_har_path | Stub only | Defined but unused |
| record_video_dir | Stub only | Defined but unused |
| record_video_size | Stub only | Defined but unused |

Missing BrowserContext methods (partial list):

| Feature | Status | Notes |
| --- | --- | --- |
| add_init_script() | Missing | Context init scripts |
| cookies()/add_cookies()/clear_cookies() | Missing | Cookie management |
| storage_state() | Missing | Auth state |
| set_geolocation() | Missing | Runtime geolocation |
| grant_permissions()/clear_permissions() | Missing | Permissions |
| set_offline() | Missing | Offline emulation |
| set_extra_http_headers() | Missing | Headers |
| route()/unroute() | Missing | Network interception |
| expose_function()/expose_binding() | Missing | JS bindings |
| tracing.start()/stop() | Missing | Tracing |
| request | Missing | APIRequest |
| clock | Missing | Time control |

## Page Coverage

Implemented:

| Feature | Status | Notes |
| --- | --- | --- |
| goto() | Implemented | Options unused |
| url() | Implemented | Current URL |
| title() | Implemented | Page title |
| content() | Implemented | HTML via JS |
| screenshot() | Implemented | PNG bytes |
| close() | Implemented | Marks closed |
| is_closed() | Implemented | Flag check |
| locator() | Implemented | CSS only |
| click()/fill()/type() | Implemented | Delegates to Locator |
| text_content()/is_visible() | Implemented | Delegates |
| wait_for_selector() | Implemented | Waits via Locator |
| evaluate()/evaluate_with_args() | Implemented | JS eval |

Missing Page methods (partial list):

| Feature | Status | Notes |
| --- | --- | --- |
| reload()/go_back()/go_forward() | Missing | Navigation helpers |
| bring_to_front() | Missing | Tab focus |
| wait_for_load_state()/wait_for_url() | Missing | Wait helpers |
| add_init_script()/add_script_tag()/add_style_tag() | Missing | Injection |
| evaluate_handle() | Missing | JSHandle |
| expose_function()/expose_binding() | Missing | JS bindings |
| get_by_*() locators | Missing | Role/text/label/etc |
| query_selector()/query_selector_all() | Missing | DOM queries |
| check()/uncheck()/select_option()/hover()/dblclick() | Missing | Input actions |
| press()/focus()/blur()/tap() | Missing | Input actions |
| set_input_files() | Missing | File upload |
| drag_and_drop() | Missing | Drag and drop |
| input_value() | Missing | Input read |
| set_viewport_size()/viewport_size() | Missing | Viewport |
| emulate_media() | Missing | Media emulation |
| route()/unroute() | Missing | Network interception |
| expect_*()/on(*) | Missing | Event waiting |
| frames()/frame()/frame_locator() | Missing | Frame APIs |
| video | Missing | Video recording |
| workers() | Missing | Web workers |
| accessibility.snapshot() | Missing | Accessibility |
| console_messages() | Missing | Console buffer |
| clock | Missing | Time control |

## Locator Coverage

Implemented:

| Feature | Status | Notes |
| --- | --- | --- |
| click() | Implemented | Basic click |
| fill() | Implemented | Clear + send_keys |
| type() | Implemented | Optional delay |
| text_content()/inner_text() | Implemented | Text |
| get_attribute() | Implemented | Attr |
| is_visible()/is_enabled()/is_checked() | Implemented | State |
| count() | Implemented | Element count |
| nth()/first()/last() | Implemented | Simplified selectors |
| wait_for() | Implemented | Visible wait |
| screenshot() | Implemented | PNG |

Missing Locator methods (partial list):

| Feature | Status | Notes |
| --- | --- | --- |
| all()/all_inner_texts()/all_text_contents() | Missing | Multi element |
| and_()/or_() | Missing | Combinators |
| filter() | Missing | Text/locator filter |
| locator()/frame_locator() | Missing | Chaining |
| get_by_*() locators | Missing | Role/text/label/etc |
| check()/uncheck()/clear()/dblclick()/hover()/tap() | Missing | Input |
| press()/press_sequentially() | Missing | Keyboard |
| select_option()/set_input_files() | Missing | Forms |
| input_value() | Missing | Read value |
| drag_to() | Missing | Drag |
| focus()/blur() | Missing | Focus |
| bounding_box() | Missing | Element rect |
| dispatch_event() | Missing | DOM events |
| evaluate()/evaluate_all()/evaluate_handle() | Missing | JS eval |
| is_editable()/is_hidden() | Missing | State |
| scroll_into_view_if_needed() | Missing | Scroll |
| highlight() | Missing | Debug highlight |
| aria_snapshot() | Missing | A11y |
| describe() | Missing | Trace annotations |

## ElementHandle Coverage

Implemented:

| Feature | Status | Notes |
| --- | --- | --- |
| click()/fill()/type() | Implemented | Basic input |
| text_content()/inner_text() | Implemented | Text |
| get_attribute() | Implemented | Attr |
| is_visible()/is_enabled()/is_checked() | Implemented | State |
| tag_name() | Implemented | Tag |
| screenshot() | Implemented | PNG |
| bounding_box() | Implemented | Rect |
| scroll_into_view() | Implemented | Scroll |
| focus() | Implemented | Best-effort |

Missing ElementHandle methods (partial list):

| Feature | Status | Notes |
| --- | --- | --- |
| query_selector()/query_selector_all() | Missing | DOM queries |
| wait_for_selector() | Missing | Wait |
| evaluate()/evaluate_handle() | Missing | JS eval |
| owner_frame()/content_frame() | Missing | Frame |
| dispatch_event() | Missing | DOM events |
| hover()/dblclick()/tap() | Missing | Input |
| check()/uncheck()/select_option() | Missing | Forms |
| set_input_files() | Missing | File upload |
| press() | Missing | Keyboard |

## Missing Classes (Entirely)

The following Playwright Python classes are not implemented in Sparkle:

- APIRequest
- APIRequestContext
- APIResponse
- APIResponseAssertions
- BrowserContextAssertions
- CDPSession
- Clock
- ConsoleMessage
- Dialog
- Download
- Error (Playwright error types)
- FileChooser
- Frame
- FrameLocator
- JSHandle
- Keyboard
- LocatorAssertions
- Mouse
- PageAssertions
- Request
- Response
- Route
- Selectors
- TimeoutError
- Touchscreen
- Tracing
- Video
- WebError
- WebSocket
- WebSocketRoute
- Worker

## Assertions

Playwright Python exposes an `expect()` API with extensive assertions. Sparkle
does not include an assertions layer.

## Priority Recommendations

P0 (core test parity):

- Network interception (route/unroute, Request/Response/Route)
- Frame and FrameLocator support
- Dialog handling
- File uploads
- get_by_* locators
- Core input actions (check/uncheck/select/hover/dblclick)

P1 (common workflows):

- Navigation helpers (reload/go_back/go_forward)
- Wait helpers (wait_for_load_state/wait_for_url)
- Keyboard/Mouse/Touchscreen classes
- Cookies/storage state
- JSHandle support
- Viewport/media emulation

P2 (advanced/debugging):

- Tracing
- Video recording
- Accessibility snapshot
- HAR recording
- CDP sessions

## Notes on WebDriver Limitations

Some Playwright features require lower-level browser control than WebDriver
provides. These areas may need CDP, WebDriver BiDi, or external tooling:

- Network interception and request mocking
- Video recording and tracing
- Console and WebSocket event streams
- Precise input dispatch and actionability checks
