export interface StatusBarItem {
  id: string;
  /** A {@link StatusKey}, or anything `getStatusConfig` falls back on. */
  status: string;
  /** Makes the segment activatable — it renders as a button instead of a plain bar. */
  onSelect?: () => void;
  /**
   * Accessible name for the activatable form. A colored bar has no text of its
   * own, and the tooltip is only `aria-describedby`, so without this the button
   * is unnameable for a screen reader and unfindable by name in a test.
   */
  label?: string;
}
