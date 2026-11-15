# field

- `select.ts` — My GPS | Example site, place search, drop a point / draw outline with live area, "Use this field" (waits as a dashed glass button with its reason; lights up green with a short sheen once a field is chosen and announces it). The panel fits laptop heights without a scrollbar (height steps; search and lede step aside while drawing); while drawing only the draft counts ("Start over", draft area). Starts on an empty map; "Use my last field" restores the saved field only when pressed.
- `search.ts` — Nominatim search (debounced, ≤1 request/s) and `debounce`.
- `field.css` — styles.
