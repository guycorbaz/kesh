# Kesh Manuals — English translations

⚠️ **Status** : English translations **not yet created**. The canonical version is French in `../fr/`.

## How to contribute a translation

See the complete guide in `../README.md` section « Traductions DE/IT/EN ».

Summary for EN :

1. Copy the canonical source file from `../fr/` :
   ```bash
   cp ../fr/admin-manual.tex admin-manual.tex
   cp ../fr/user-manual.tex user-manual.tex
   cp ../fr/marketing-brochure.tex marketing-brochure.tex
   ```
2. Adjust the preamble to load English babel :
   ```latex
   \usepackage[english]{babel}
   ```
3. Translate the entire content (titles, sections, paragraphs, lists, table captions).
4. Keep technical references unchanged (variable names, file paths, shell commands).
5. Run `cd .. && make all-langs` to verify compilation.
6. Have it reviewed by a native EN speaker (reference Story 9-1 L4 v0.2).

## Planned translation stories

DE/IT/EN translations will be delivered as separate stories, scheduled for Epic 10+ or a dedicated Epic « Internationalisation Documentation v0.2 ». See project roadmap in `_bmad-output/implementation-artifacts/sprint-status.yaml`.
