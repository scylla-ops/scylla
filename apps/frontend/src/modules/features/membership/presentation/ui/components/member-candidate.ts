/**
 * The little a candidate picker needs: both member lists already carry it.
 *
 * In a `.ts` rather than the dialog's `<script>`: `tsc` sees only a `.svelte`
 * file's default export, so a type another file imports cannot live in one.
 */
export interface MemberCandidate {
  userId: string;
  username: string;
}
