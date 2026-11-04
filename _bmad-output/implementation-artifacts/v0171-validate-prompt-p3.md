# Prompt — validation P3 de la fiche v0171-correctifs (release corrective 0.17.1), passe ciblée, une lentille, Opus

Tu valides la **fiche** de la Story v0171-correctifs de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) — release corrective
0.17.1 (#854, #866, #871, #878, #879), sans migration. Worktree `/home/gcorbaz/devel/kesh-0171`, branche `release/0.17.1`, partie de
`910e5d5a`. Langue du rapport : **français**. Modèle : **Opus**. Passe **ciblée** : **une seule lentille**, braquée sur **le seul commit de
la remédiation de la validation P2** et sur **le seul inventaire des sites `unitPrice` / `quantity`**. Ce n'est pas une revue de la fiche entière
(deux passes complètes ont eu lieu). **Contexte frais.**

⛔ **Piège de l'environnement** : dans ce shell, `grep` est une **fonction** qui appelle `ugrep` (avec `--ignore-files`) et peut rendre autre chose que
GNU grep. Pour toute citation décisive, utilise `/usr/bin/grep` ou `git grep`, et dis-le. Un motif qui rend « rien » se prouve par une sonde positive.

## À lire

1. `git log --oneline -4`, puis `git show <commit de la remédiation P2>` (titre : `docs(v0171) : remédiation de la validation P2 (refs #878)`) — **c'est ton
   périmètre**. Le rapport de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/v0171-validate-p2.md` (1 MEDIUM, 4 LOW).
2. La fiche `_bmad-output/implementation-artifacts/v0171-correctifs.md` : § 11, AC-2 (d), AC-4, AC-8, AC-11, AC-12, T13, M6 à M6-ter, tâches 5 et 8,
   § CHANGELOG, Change Log (entrées P1 et P2) ; le registre `v0171-choix-autonomes.md` (C-v0171-13, -19, -20) ; `sprint-status.yaml` (clé `v0171-correctifs`).
3. Le code : `frontend/src/lib/components/invoices/{InvoiceForm.svelte,ProductPicker.svelte,InvoiceForm.test.ts}`,
   `frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts`, `frontend/src/routes/(app)/products/+page.svelte`,
   `crates/kesh-api/src/routes/invoices.rs` (`line_scale_error`, `MAX_DECIMAL_SCALE`), `crates/kesh-db/migrations/20260415000001_products.sql`,
   `20260416000001_invoices.sql`.
4. Issues, en lecture seule : `gh issue view 878 --comments`, `gh issue view 880`.
5. Contexte : `CLAUDE.md` du worktree (*La passe ciblée*, *Propagation post-patch*, *Recompter ses propres comptes rendus*) ;
   `docs/agents-orchestration.md`.

## Axes à exercer (et seulement ceux-là)

- **L'inventaire clos des sites `unitPrice` / `quantity`** (fiche § 11) : refais-le **toi-même**, plus large que la fiche
  (`git grep -nE '(unitPrice|quantity|unit_price|Qté|Prix)' frontend/src`, hors tests ; affectations `.unitPrice =`, `.quantity =`, spread d'objets de ligne,
  `push({…})`, `structuredClone`, import de lignes, duplication de facture, création d'avoir depuis une facture, copie d'une facture, articles, modèles de
  facture, relances, devis). **Sonde positive** pour chaque motif. Un site qui pose un prix ou une quantité **lus du serveur** dans un champ de saisie, et que
  la fiche omet ? Chaque ligne « conforme » l'est-elle vraiment (lis le code, ne te fie pas au numéro de ligne) ? `products/+page.svelte:308-310` : le champ
  prix d'un article est-il vraiment déjà ramené à deux décimales dans **tous** ses chemins (création, édition, duplication) ?
- **Trois sites pour le prix, deux pour la quantité** : la propagation de ce compte est-elle complète — § 11, tâche 5, AC-8, AC-11, T13, M6-ter, CHANGELOG,
  C-v0171-19, Change Log, `sprint-status.yaml` ? **Greper la valeur** (`deux sites`, `2 sites`, `trois sites`, `:114`, `:425`, `:611`, `onProductSelect`)
  sur la fiche, le registre, le CHANGELOG prescrit et ce prompt ; les occurrences restantes sont-elles légitimes (quantité à deux sites ; entrées datées
  de l'historique, qui ne se réécrivent pas — seulement s'annotent) ?
- **Le correctif prescrit pour `onProductSelect`** : `readableAmount(p.unitPrice, 2)` est-il sûr (type de `ProductResponse.unitPrice`, valeur nulle ou absente
  possible, article sans prix) ? Le test prescrit (ajout depuis un article de prix `100.0000` ⇒ `100.00`) est-il réalisable avec le banc de
  `InvoiceForm.test.ts` (comment `ProductPicker` est-il monté et appelé) ? M6-ter, **une mutation par site**, est-elle tuable pour chacun des trois sites, ou l'un
  d'eux est-il couvert par un test qui reste vert sous la mutation ? L'aller-retour (`"100.00"` envoyé pour `100.0000` lu) reste-t-il sans effet sur le total et sur
  la validation serveur (`line_scale_error`, `MAX_UNIT_PRICE`) ?
- **Les quatre LOW de la P2** : P2 (`sprint-status.yaml` : P1/P2 remédiées, prochaine P3, #880) ; P3 (AC-12 : les cinq lignes d'exceptions `[id]:1000, :1014,
  :1561, :1583`, `due-dates:514` ; le motif ne rend-il bien que ces cinq lignes une fois les dix-sept sites enveloppés ? joue-le mentalement sur un site
  enveloppé) ; P4 (AC-2 d : `DecodeError::InvalidSpcPayload(_)`, payload de 4 lignes, `parser.rs:90-100`) ; P5 (`spc_payload` `:105`). Chacune est-elle exacte ?
- **Régression de la remédiation** : le commit a-t-il introduit une incohérence (compte, renvoi, numéro de choix, de test ou de mutation, ligne citée) ? Le
  registre porte-t-il toujours C-v0171-1 à 20, cité de même partout ? L'entrée P2 du Change Log est-elle un relevé daté, sans réécriture d'entrée passée ?
- **Hors périmètre de cette passe** (ne les rouvre pas, signale seulement un défaut flagrant) : le fond des AC-1 à AC-7, le manuel, le CHANGELOG de #854, les
  mutations M1 à M5 et M7.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0171-validate-p3.md` :
1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu, **la ligne lue citée** (sortie de
   `git grep -n` ou `/usr/bin/grep -nF`, copiée) qui fonde l'affirmation, et le remède. Une affirmation de présence ou d'absence sans sortie copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`, `tee`) — pas même vers un fichier
  « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` et `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- `grep` du shell est la fonction `ugrep` : n'en tire aucune citation décisive (`/usr/bin/grep`, `git grep`).
- Commandes permises : `cat`, `sed -n`, `grep` (binaire GNU), `git grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff` (en
  lecture), `gh issue view`, `gh api` en `GET`.
