# Prompt — revalidation R7 CIBLÉE, Story 25-1c-b2

*Versionné le 2026-09-26. Une lentille (Haiku 4.5), contexte frais, braquée sur la seule remédiation de
la R6 — CLAUDE.md, § « La passe ciblée ». Diff APLATI.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-1c-b1-journal-audit-ecran`. Diff à relire :
`git diff ef575755 5291c693 -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`.
Fiche : `_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md` (l'entrée « revalidation
R6 » en tête du Change Log dit ce qui a été corrigé).

## Ce que tu vérifies

1. **Chaque ligne AJOUTÉE à l'inventaire de l'AC 3** (`user-manual.tex:301`, `:1803-1812`,
   `admin-manual.tex:1760`, `:1766`, `:1816`, `:1946`, `:1969`, `README.md:220`) : le texte cité est-il à
   cette ligne (`grep -nF` d'une portion de la phrase), et le remplacement juste ?
2. **Le repli Svelte** de l'AC 2 : `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` porte-t-il
   bien « piste d'audit » à la ligne dite ?
3. **La réserve de l'AC 4 et de l'AC 6** (« en usage courant, hors restauration d'une sauvegarde ») : vraie
   contre `crates/kesh-db/src/backup.rs` (effacement des tables à l'import) et `crates/kesh-seed` ?
4. **Le symptôme ailleurs dans la fiche** : reste-t-il une phrase qui dit « quatre énoncés », « la seule »
   sans réserve, ou qui classe `:1760` / `:1946` autrement ? Le rejeu de l'AC 8 et l'inventaire de l'AC 3
   sont-ils d'accord entre eux (un site trié « autre sens » dans l'un et « à changer » dans l'autre) ?
5. **Rejoue le grep lexical** de l'AC 3 sur `docs/manual/fr/*.tex` et `README.md` : chaque ligne rendue
   est-elle, dans la fiche, soit à changer, soit déjà conforme, soit autre sens ?

## Ce que tu rends

- Findings : sévérité, endroit, **preuve** (commande et résultat) ; pour tout CRITICAL ou HIGH, la commande
  `grep -nF` et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier ; aucune commande qui écrit dans le dépôt ou une base (`scripts/*.sh`, `make`,
`git commit`/`add`/`checkout`/`reset`/`stash`, `cargo test`, `npm run`, `npx`). Lecture, `grep`,
`git diff`/`show` seulement.
