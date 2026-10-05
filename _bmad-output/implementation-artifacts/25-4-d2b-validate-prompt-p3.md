# Prompt — validation P3 ciblée, Story 25-4-d2b (le bouton, le dialogue et la liste du solde)

*Versionné le 2026-10-02. **Une lentille** (Sonnet), contexte frais — passe ciblée sur la remédiation de P2. Trend :
P1 2H/3M/2L (Sonnet) → P2 9M/8L (Opus).*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2b-bouton-solder-le-reste`. **Diff à relire : `git diff 62f01348 6be35c39`**
(la fiche `_bmad-output/implementation-artifacts/25-4-d2b-bouton-solder-le-reste.md`). La fiche décrit du travail **à
faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

Cherche ce que **cette remédiation** a cassé ou laissé incohérent :
- la fiche dit-elle encore la même chose partout (AC 2, AC 3, AC 4, AC 6, AC 7, *Limites assumées*, *Ce qu'il ne faut
  pas faire*, tâches) ? Une règle ajoutée à un endroit est-elle contredite ailleurs ?
- chaque **référence** ajoutée (`errors.rs:2841-2847`, `invoice_email.rs:885`, `+page.svelte:190-198`, `:247-251`,
  `lib.rs:207-211`, `CHANGELOG.md:49`, `:79`, `user-manual.tex:1063-1064`, `admin-manual.tex:2009`,
  `i18n-un-repli-par-cle.test.ts`) existe-t-elle et dit-elle ce que la fiche affirme ?
- les règles nouvelles sont-elles **réalisables** et cohérentes entre elles : la fermeture du dialogue « après toute
  relecture » contre la consigne P1 « le dialogue reste ouvert après un 409 » ; le drapeau « liste chargée » contre
  l'affichage pendant le chargement ; les quatre décimales à l'affichage contre « au centime » ; « relire la facture
  après l'envoi d'un e-mail » — est-ce dans le périmètre (deux modules) et nommé en tâche ?
- le montage de l'E2E : le 4000 du seed est-il **imputable** et de type charge ? Le `PUT` des réglages exige-t-il
  d'autres champs que l'E2E devra renvoyer ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande et sortie). Pour tout finding affirmant qu'un code est absent ou
présent : la sortie d'un `grep -nF`. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git log`/`show`/`diff`,
`gh issue view`.
