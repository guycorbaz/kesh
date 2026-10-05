# Prompt — validation P3 ciblée, Story 25-4-d2c

*Versionné le 2026-10-02. **Une lentille** (Sonnet), contexte frais — passe ciblée sur la remédiation de P2. Trend :
P1 1H/1M/2L (Sonnet) → P2 1H/3M/5L (Opus).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2c-rapport-tva-soldes`. **Diff à relire : `git diff 2ca5937a 00b76947`**
(la fiche `_bmad-output/implementation-artifacts/25-4-d2c-rapport-tva-soldes.md`). La fiche décrit du travail **à faire** :
un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

- **Cohérence interne** : la fiche dit-elle la même chose partout (AC 1 à AC 8, tâches, Dev Notes, *Modules*) après
  ce patch ? En particulier : « TVA due nette seulement s'il y a des soldes » (AC 4) contre la phrase de l'AC 2 et les
  rendus CSV/PDF — le CSV et le PDF l'affichent-ils toujours, ou eux aussi seulement avec des soldes ? Le test « rapport
  vide » de l'AC 7 est-il cohérent avec les trois gardes ?
- **Les références ajoutées** existent-elles et disent-elles ce que la fiche affirme : `reports.spec.ts:234-237`,
  `invoice_settlements_write.rs:767`, `user-manual.tex:1656`, `:1666`, `:1670-1680`, `:2125`, `README.md:44`,
  `kesh-api/src/errors.rs:931-975`, `pdf.rs:1004`, `pdf.rs:1080`, `vat.rs:193-198` ?
- **`ReportError::CorruptData` → 500** : la conversion `From<ReportError> for AppError` est-elle bien un `match`
  exhaustif (donc l'ajout d'une variante est obligatoirement traité) ? Le 500 doit-il porter un code d'erreur et une clé
  i18n, selon les conventions de `errors.rs` ?
- **L'E2E étendu** (`invoice-write-off.spec.ts`, rapport TVA après le solde) : la spec annule le solde à la fin — le
  rapport doit donc être vérifié **avant** l'annulation ; la fiche le dit-elle ? La période du rapport (exercice courant,
  sélection par défaut de l'écran) contient-elle la date du solde ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie, `grep -nF` pour toute présence ou absence). ⛔ La liste
des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`.
