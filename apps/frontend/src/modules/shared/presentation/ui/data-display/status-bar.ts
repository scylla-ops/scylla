export interface StatusBarItem {
  id: string;
  status: string;
  /** Renders the segment as a button. */
  onSelect?: () => void;
  /** The accessible name of the button: a colored bar has no text. */
  label?: string;
}
