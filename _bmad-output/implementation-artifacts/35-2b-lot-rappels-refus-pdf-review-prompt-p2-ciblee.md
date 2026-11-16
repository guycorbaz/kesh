# Prompt — revue de code P2 ciblée de la Story 35-2b (#955), une lentille, braquée sur la remédiation de la P1

Passe **ciblée**, une seule lentille, contexte frais, **Sonnet** (P1 : Opus), en lecture seule. Worktree `/home/gcorbaz/devel/kesh-e352b`,
branche `story/35-2b-lot-rappels-refus-pdf`. La P1 (trois lentilles) a rendu 2 MEDIUM et 10 LOW (rapports
`/home/gcorbaz/devel/kesh-gate-logs/e352b-review-p1-{B,E,A}.md`). Déjà tracés, à ne pas rapporter de nouveau : #1108 (jointure « : »,
langues mêlées ; B4 en commentaire), #1110 (phrase « scindez la facture » impossible sur une facture validée).

## Périmètre : le seul commit `eb512ea7a`

`git -C /home/gcorbaz/devel/kesh-e352b show eb512ea7a` — `errors.rs` (renommage d'un test), `invoice_send_email_e2e.rs` (test HTTP neuf
`send_reminder_batch_mixte_trop_de_lignes_porte_le_message`), `textes_coherents.rs` (G9), `docs/api-external.md`, `user-manual.tex` et
son PDF, `reminder-error-label.ts` (commentaire). Compte rendu : Change Log et Dev Agent Record de la fiche
`_bmad-output/implementation-artifacts/35-2b-lot-rappels-refus-pdf.md` (commit `710a07030`). Mutations R1 à R4 :
`/home/gcorbaz/devel/kesh-gate-logs/e352b/mut/R*.log`, `campagne-rem1.log`.

Il remédie :
- **E1/A2 (MEDIUM)** — le manuel promettait le motif du refus au lot « comme sur la fiche de la facture » ; or le PDF de la facture est
  figé au premier rendu (`get_or_freeze`), le rappel régénéré à chaque envoi : un rappel peut être refusé alors que la fiche sert son PDF
  sans refus.
- **E2 (MEDIUM)** — l'angle mort « trop de lignes au lot, prouvé par le repli seul » reposait sur une prémisse fausse (le pré-filtre
  `lines.len() > MAX_LINES_PER_PDF`, `invoice_pdf_service.rs:206`, précède tout rendu) ; test HTTP neuf. Ferme #1109.

## Ce que tu vérifies

1. **La phrase du manuel** (`user-manual.tex:1527` et la phrase qui suit) : vraie au code ? L'envoi d'un rappel seul et le lot passent-ils
   par le même rendu et la même phrase (`invoice_email.rs:560`, `:1260`, `refus_pdf_au_lot`) ? « le PDF du rappel est produit à chaque
   envoi », « la fiche de la facture montre sans erreur le PDF émis » : vrai pour une facture jamais figée (premier téléchargement après
   un refus de rappel) ? Le PDF aplati porte-t-il la phrase (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` puis
   `/usr/bin/grep -o`) ? Reste-t-il ailleurs (manuels, CHANGELOG, `api-external.md`, commentaires, fiche hors § Story) une promesse
   « comme sur la fiche de la facture » ou « ce qu'il y a à corriger » au lot ?
2. **Le test neuf** : sa facture de dix lignes est-elle **validée** (sinon le refus serait `InvoiceNotValidated`, pas « trop de lignes »)
   ? Asserte-t-il `details` **entier** (égalité d'objet, pas inclusion), la phrase avec `count` = 10, l'absence de U+2068/U+2069, l'envoi
   réussi dans `accepted`, `details` nul pour la facture payée ? Le catalogue réel est-il chargé (sinon il prouverait le repli, l'angle
   mort même) ? Les mutations R1 à R3 sont-elles tuées **par ce test**, nommé au journal ? Le montage est-il synthétique, isolé des autres
   tests du binaire (contact, numéro de facture, quota d'envoi) ?
3. **G9** (`textes_coherents.rs`) : l'entrée ajoutée pour `error-invoice-too-many-lines-for-pdf` compte-t-elle le bon nombre de sites ;
   R4 (littéral recopié dans `IntoResponse`) tué par G9 ?
4. **`api-external.md`** : les deux lignes ajoutées (`INVOICE_TOO_MANY_LINES_FOR_PDF`, `INVOICE_PDF_HEADER_OVERFLOW`) disent-elles vrai :
   statut HTTP, routes, corps (`details` au lot, message à la route unitaire) — lis `errors.rs` (bras de `IntoResponse`) et les routes.
5. **Renommage B2** : le nouveau nom dit-il ce que le test prouve ? Aucune référence à l'ancien nom ne reste (`git grep -n
   'route_unitaire_et_lot_meme_phrase'`) dans la fiche, les journaux cités, les commentaires ?
6. **Les annotations A1, A3, A4, A5** : datées, sans réécriture (`git show 710a07030`, lignes `-`) ; A4 : `sitesTotal` (`i18n-keys.test.ts`)
   recompté et inchangé, ligne lue ?
7. **Gates** au commit `eb512ea7a` : test-fast 5335, frontend complet, `make user` — chacun au journal (`kesh-gate-logs/e352b/rem1-*.log`),
   postérieur au commit ; `gate-ci` et E2E déclarés « à la clôture » (règle du 2026-11-13, `kesh-gate-logs/prompts/e35-dev-commun.md`),
   non affirmés.

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, **aucune redirection** (`>`, `>>`, `tee`, `2>`, `2>&1`, `2>/dev/null`,
`> /dev/stdout`, `> /dev/null` compris), aucun fichier temporaire. Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur
ni script (`python3`, `perl`, `node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les
scripts de `kesh-gate-logs`, `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`),
aucune commande `gh`, aucun accès réseau. Ne lis rien sous `~/kesh/` ni `~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`,
`/usr/bin/grep`, `sed -n`, `awk` sans redirection ni `system()`, `cat`, `ls`, `find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`, `cmp`,
`pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`. ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`. Toute
affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un écart se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e352b-review-p2-ciblee.md` : findings numérotés (`C1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
