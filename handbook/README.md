# handbook

Team documentation in two languages. English is the source of truth and the default view.

| Path | What it is |
|------|------------|
| `en/NN-slug.md` | English pages. The file name sets the order and the page id. |
| `es/NN-slug.md` | Spanish mirror. Same file name as the English page. |
| `build.mjs` | Bundles both folders into `index.html`. Warns about missing translations. |
| `site/template.html` | Page shell with placeholders for CSS, data and JS. |
| `site/viewer.css` | Styles, light and dark. |
| `site/viewer.js` | Navigation, EN/ES switch (saved in localStorage), Markdown rendering via marked. |
| `index.html` | Generated. Do not edit by hand. |

Rules: edit both languages in the same change, link pages as `NN-slug.md`, then run `node handbook/build.mjs`.
