// What a screen reader is handed. A port of
// `apps/android/surfaces/shared/src/main/kotlin/streams/shared/Says.kt`, whose reasoning holds
// unchanged here: that file records Home drawing 61 nodes on a device, 19 of them reachable, and
// "all 19 carried neither text nor a description" before any of it existed, with the word "Play"
// sitting one node away from the Play button where nothing would ever focus it.
//
// ArkUI says the same three things with two attributes instead of a modifier, and both are
// verified against `ts-universal-attributes-accessibility` (the module is API 10):
//
//   saysItsWords()   ->  .accessibilityGroup(true)
//                        "When accessibility grouping is enabled, the component and all its
//                        children are treated as a single selectable unit, and the accessibility
//                        service will no longer focus on the individual child components" -- and,
//                        for a component with no words of its own, "the system will concatenate the
//                        universal text attributes of its child components to generate merged text".
//                        That is exactly `semantics(mergeDescendants = true)` plus the Compose
//                        behaviour the Kotlin relies on, and it needs no key.
//   says(label)      ->  .accessibilityGroup(true).accessibilityText(label)
//                        "If a component lacks text content, you can set the accessibility text
//                        attribute... If a component already contains text content, the
//                        accessibility text takes precedence."
//   says(l, state)   ->  the same, plus .accessibilityDescription(state), which is announced after
//                        the text content -- Compose's `stateDescription`.
//   decoration()     ->  .accessibilityLevel(SKIP_ALL)
//
// There is no helper here that wraps those calls. `@Styles` cannot take a parameter and `@Extend` is
// bound to one component type, so a wrapper would have to be an `AttributeModifier`, and the
// reference says of several of these attributes that they "cannot be called within
// attributeModifier". So every call site writes the chain, and this file holds only the two level
// words so that neither is ever mistyped.
//
// NOT COMPILED. See ../../OPEN-QUESTIONS.md.

/**
 * This node is decoration, and so is everything inside it: a gradient, a plate, the art inside a
 * tile that already says the title's name, the brand's mark beside the page's own name.
 *
 * `no-hide-descendants`, not `no`: the reference defines `no` as "the component cannot be
 * recognized" while `no-hide-descendants` is "Neither the component nor its child components can be
 * recognized". A decorative container whose children were still reachable is the same defect as not
 * marking it -- a reader stops on the thing twice.
 */
export const SKIP_ALL: string = 'no-hide-descendants';

/**
 * This node alone is skipped, and what is inside it is not.
 *
 * Used where a node exists only to hold a layout and its children are the content.
 */
export const SKIP_ONE: string = 'no';

/**
 * Read this even though an ancestor is grouped.
 *
 * The reference: "When a child component's accessibilityLevel is set to 'yes', it becomes focusable
 * by screen readers ... bypassing accessibilityGroup constraints." It is what the two buttons inside
 * the billboard need: the card is one control that opens the title and says the title's name, and
 * the buttons inside it are reached separately with their own words
 * (`browse/Billboard.kt:121-124`).
 */
export const READ_ANYWAY: string = 'yes';
