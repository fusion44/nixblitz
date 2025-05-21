// use dioxus::prelude::*;
// use web_sys::wasm_bindgen::prelude::Closure;
//
// // Define the props for our DropDown component
// #[derive(Props, PartialEq, Clone)]
// pub(crate) struct DropDownProps {
//     // The text to display on the dropdown button
//     label: String,
//     // The content to be rendered inside the dropdown. This allows for flexible options.
//     children: Element,
//     // Optional: A class string to apply to the main dropdown container for custom styling.
//     #[props(default)]
//     class: String,
//     // Optional: A class string to apply to the dropdown button.
//     #[props(default)]
//     button_class: String,
//     // Optional: A class string to apply to the dropdown content container.
//     #[props(default)]
//     content_class: String,
// }
//
// /// A reusable DropDown component for Dioxus.
// ///
// /// This component displays a button that, when clicked, toggles the visibility
// /// of a content area. The content area automatically closes when a click
// /// occurs outside of the dropdown.
// ///
// /// # Props
// /// - `label`: The text to display on the dropdown button.
// /// - `children`: The content to be rendered inside the dropdown (e.g., `ul` with `li` items).
// /// - `class`: Optional. A class string to apply to the main dropdown container.
// /// - `button_class`: Optional. A class string to apply to the dropdown button.
// /// - `content_class`: Optional. A class string to apply to the dropdown content container.
// ///
// /// # Example Usage
// /// ```rust
// /// rsx! {
// ///     DropDown {
// ///         label: "Select an Option".to_string(),
// ///         children: rsx! {
// ///             ul {
// ///                 li { class: "p-2 hover:bg-zinc-700 cursor-pointer", "Option 1" }
// ///                 li { class: "p-2 hover:bg-zinc-700 cursor-pointer", "Option 2" }
// ///                 li { class: "p-2 hover:bg-zinc-700 cursor-pointer", "Option 3" }
// ///             }
// ///         }
// ///     }
// /// }
// /// ```
// #[component]
// pub fn DropDown(props: DropDownProps) -> Element {
//     // State to manage whether the dropdown is open or closed
//     let mut is_open = use_signal(|| false);
//
//     // Ref to the main dropdown container element. This is used to detect clicks outside.
//     let node_ref = use_node_ref();
//
//     // Effect to handle closing the dropdown when clicking outside of it.
//     use_effect(move || {
//         let document = web_sys::window().unwrap().document().unwrap();
//         let node_ref_clone = node_ref.clone(); // Clone for the closure
//
//         // Define the click handler
//         let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
//             // Check if the click occurred outside the dropdown
//             if let Some(element) = node_ref_clone.get() {
//                 if !element.contains(&event.target().unwrap()) {
//                     // If clicked outside, close the dropdown
//                     is_open.set(false);
//                 }
//             }
//         });
//
//         // Add the event listener to the document
//         document
//             .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
//             .expect("Failed to add event listener");
//
//         // Store the closure in a RefCell to prevent it from being dropped
//         // This is a common pattern in Dioxus for managing JS closures.
//         let closure_ref = Rc::new(RefCell::new(Some(closure)));
//
//         // Return a cleanup function that removes the event listener when the component unmounts
//         to_owned![closure_ref, document];
//         on_drop(move || {
//             if let Some(closure) = closure_ref.borrow_mut().take() {
//                 document
//                     .remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
//                     .expect("Failed to remove event listener");
//             }
//         });
//     });
//
//     rsx! {
//         // Main container for the dropdown. Positioned relative for absolute positioning of content.
//         div {
//             class: "relative inline-block {props.class}",
//             // Attach the node_ref to this div to detect outside clicks
//             node_ref: node_ref,
//             tabindex: 0, // Make it focusable for accessibility
//
//             // The dropdown button
//             button {
//                 class: "inline-flex justify-center items-center px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 {props.button_class}",
//                 onclick: move |_| is_open.set(!*is_open.read()), // Toggle dropdown visibility
//                 "{props.label}"
//                 // Optional: Add a small arrow icon
//                 svg {
//                     class: "-mr-1 ml-2 h-5 w-5",
//                     xmlns: "http://www.w3.org/2000/svg",
//                     view_box: "0 0 20 20",
//                     fill: "currentColor",
//                     "aria-hidden": "true",
//                     path {
//                         "fill-rule": "evenodd",
//                         d: "M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z",
//                         "clip-rule": "evenodd",
//                     }
//                 }
//             }
//
//             // The dropdown content, conditionally rendered based on `is_open`
//             // Positioned absolutely to overlay other content
//             if *is_open.read() {
//                 div {
//                     class: "origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-zinc-800 ring-1 ring-black ring-opacity-5 focus:outline-none {props.content_class}",
//                     role: "menu",
//                     "aria-orientation": "vertical",
//                     "aria-labelledby": "menu-button",
//                     tabindex: -1, // Make it not focusable by tab, but programmatically focusable
//
//                     div {
//                         class: "py-1",
//                         role: "none",
//                         {props.children} // Render the children passed to the component
//                     }
//                 }
//             }
//         }
//     }
// }
