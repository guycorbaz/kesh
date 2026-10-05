# Prompt — revue de code P3, Story 25-6-b (le PDF d'une facture est figé)

*Versionné le 2026-10-05. Deux lentilles (Sonnet), contexte frais chacune. Rotation : P1 Sonnet ×3 → P2 Opus ×2 → P3
Sonnet ×2.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. Fiche :
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md` — lire le **Change Log** (entrées « Revue de code
P1 » et « P2 »). Règles : `CLAUDE.md`.

**Constat qui motive la passe** : en P2, deux des cinq MEDIUM venaient de la remédiation P1. La remédiation P2
(`b2eeaab9`) touche encore la production en plusieurs modules : `errors.rs` (`t_args`, `InvoicePdfGone`),
`routes/issued_invoice_pdf.rs` (`Usage`, relecture par tentative), `routes/invoice_pdf.rs`, `routes/invoice_email.rs`,
`frontend/src/lib/shared/utils/pdf-error.ts` (nouveau), les fiches facture et avoir.

## Lentilles

- **R — Regression hunter** : **`git show b2eeaab9` seul**, puis le code qu'il touche.
  - `t_args` retire désormais U+2068/U+2069 de **tous** les messages d'erreur à argument : un appelant comptait-il
    sur elles (test, frontend, texte bidirectionnel) ? `grep -n "t_args(" crates/kesh-api/src/errors.rs`.
  - `InvoicePdfGone { sha256, refreezable }` : chaque site de construction donne-t-il la bonne valeur ? un refus
    `refreezable = true` sur une facture annulée, ou l'inverse, est-il possible ?
  - `Usage::Send` : la garde est-elle avant ou après le service ? double garde cohérente avec celle du handler ?
  - `render_and_pose` relit la facture et compare `rendered.invoice_version` : une boucle infinie, un `Changed`
    systématique (version lue par `render_document` différente pour une raison structurelle) ?
  - `pdfErrorMessage` : chaque code des anciennes tables est-il conservé (`git show b2eeaab9^:…` pour les deux
    fiches) ? un message qui était traduit côté client et l'est désormais côté serveur — quelle langue ?
  - La fiche relue après refigeage : un échec de la relecture laisse-t-il l'écran dans un état faux (refigeage
    réussi, erreur affichée) ?
  - **Propagation** : le symptôme R2-1 (variable brute d'un message tiré du catalogue) existe-t-il ailleurs que
    sur le PDF ? Pars du symptôme : les clés `.ftl` à variable (`grep -n "{ \$" crates/kesh-i18n/locales/fr-CH/messages.ftl`)
    croisées avec les `i18nMsg(<clé>, …)` **sans** troisième argument dans `frontend/src`.
- **C — Acceptance auditor** : l'état `b2eeaab9`, chaque AC (1 à 8) contre le code.
  - Recompte depuis la source : `sitesTotal` (1765), `sitesNonResolus` (33), parité des clés neuves dans les 4
    `.ftl`, décomptes de tests des trois entrées du Change Log, compteurs des registres (bloc admin 28, routes
    110/92/113), migrations 71.
  - **Le manuel** : `docs/manual/fr/user-manual.tex` et `admin-manual.tex` **et leurs PDF aplatis**
    (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`). Le texte dit-il ce que fait le code —
    notamment le 410 d'une facture annulée, `HEAD`, la clé d'API, la restauration, l'export ? Pars du symptôme :
    `grep -nE "PDF|figé|refig|410" docs/manual/fr/*.tex`.
  - Le CHANGELOG `[0.12.1]` : dit-il les changements visibles de P1 et P2 (envoi d'une facture annulée refusé,
    message du 410) ?
  - Les « non traités » déclarés au Change Log (communication QR en français, fiche non relue après un envoi
    par e-mail, F-L1 sans test) : exacts ? acceptables ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code cité relu),
scénario d'échec, correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la sortie d'un
`grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
