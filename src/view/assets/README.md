# Reader assets

The reader embeds these resources into its shared live/static HTML shell. There are
no remote logo, decoration, math-script or math-font requests.

- `logo.svg`: new project-native geometric M/book mark, olive and champagne gold.
  `logo.uri` is its base64 data URI for favicon and wordmarks.
- `blossom.png`, `feather.png`: original transparent artwork generated with the
  built-in imagegen tool for this redesign. Their WebP copies are resized browser
  assets; `.uri` files contain the same WebP bytes as base64 data URIs.
- `vendor/`: KaTeX 0.16.22, reused from the workspace's existing local distribution.
  See `vendor/KATEX-LICENSE`. CSS fonts are embedded as woff2 data URIs, so opening
  a static export from disk also renders formulas offline.

## Final generation prompts

### Blossom

Use case: illustration-story. Asset type: transparent corner botanical ornament for Marqdo literary executable-document reader. A single elegant cherry blossom branch entering from upper left, thin olive and muted gold stems, delicate ivory and pale blush five-petal blossoms, sparse small olive leaves. Refined East Asian botanical watercolor with very fine warm gold ink contours, airy asymmetrical composition, most flowers concentrated in upper-left quadrant with branch tapering toward lower right. True transparent background, no paper rectangle, no text, no logo, no UI, no shadow. Soft quiet sophisticated editorial style, readable content will be placed beside this ornament. Square canvas.

### Feather

Use case: illustration-story. Asset type: small elegant quill ornament for a literary document reader wordmark. One single fine feather, curved stem rising diagonally from lower left to upper right, graceful tapered silhouette, warm pale champagne gold watercolor wash with delicate gold ink veins, restrained antique engraving style, no pen nib, no text, no shadows, no other objects. True transparent background. Portrait composition with clear edges, entire feather visible, sophisticated quiet airy appearance.

## Updating embedded resources

After replacing a project asset, regenerate its URI from its bytes (base64 with no
line wrapping, prefixed with `data:image/webp;base64,` or
`data:image/svg+xml;base64,`). Keep PNG originals for future design edits. No
runtime asset server or extra Cargo dependency is required.
