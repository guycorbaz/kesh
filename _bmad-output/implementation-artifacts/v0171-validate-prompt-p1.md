# Prompt — validation P1 de la fiche v0171-correctifs (release corrective 0.17.1 : #854, #866, #871), une lentille complète R+F, Opus

Tu valides la **fiche** de la Story v0171-correctifs de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), une
**petite** story de release corrective, **sans migration** : **#854 (P2)** — les QR-factures dont les lignes sont séparées par
CR+LF sont écartées par le préfiltre `SPC_HEADER = "SPC\n0200"` de `first_spc`
(`crates/kesh-api/src/qr_decode.rs:31, :261`) alors que le parseur les tolère (`crates/kesh-qrbill/src/parser.rs:95`) ;
**#866 (P4)** — la quantité d'une ligne s'affiche brute (`1.0000`) ; **#871 (P4)** — des dates ISO là où la forme suisse est attendue
(la fiche soutient que la prémisse de l'issue est **fausse** pour la confirmation d'annulation de règlement : à contrôler).

Worktree `/home/gcorbaz/devel/kesh-0171`, branche `release/0.17.1`, partie de `910e5d5a`. Langue du rapport : **français**.
Modèle : **Opus**. Passe **complète**, première passe de validation, **une seule lentille** qui tient à la fois R (la règle et sa
preuve) et F (les textes et ce qu'ils affirment). **Contexte frais** : ne t'appuie sur aucune passe antérieure, ni sur ce que la
fiche dit d'elle-même ; refais les relevés.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v0171-correctifs.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v0171-choix-autonomes.md` (C-v0171-1 à 12).
3. Les issues, **en lecture seule** : `gh issue view 854 --comments`, `gh issue view 866`, `gh issue view 871` (si `gh issue view`
   échoue sur « Projects (classic) », `gh api repos/guycorbaz/kesh/issues/<n>/comments` en lecture).
4. Le code : `crates/kesh-api/src/qr_decode.rs` (en entier), `crates/kesh-api/src/qr_decode_synthetique.rs` (`payload()`,
   `donnees_facture()`), `crates/kesh-qrbill/src/{parser,generator,lib}.rs`, `crates/kesh-api/src/inbox_import.rs`
   (appels `decode_spc_from_*`), `crates/kesh-api/src/routes/supplier_invoices.rs` (`scan_qr_supplier_invoice`, `scan_qr_tests`),
   `crates/kesh-api/tests/{inbox_import_e2e,supplier_invoice_scan_qr_e2e}.rs`,
   `frontend/src/lib/features/supplier-invoices/{supplier-invoice-scan,supplier-invoice-form}.ts`,
   `frontend/src/lib/features/reports/reports.api.ts` (`formatSwissDate`),
   `frontend/src/routes/(app)/{supplier-invoices,payment-batches,invoices,credit-notes}/…/+page.svelte`,
   `frontend/src/lib/components/invoices/InvoiceForm.svelte`,
   `frontend/src/routes/(app)/supplier-invoices/[id]/supplier-settlement-page.test.ts`.
5. Le **manuel** : `docs/manual/fr/user-manual.tex` (§ import des factures fournisseur, `:1965-2030`, et partout où la lecture du
   QR, la quantité ou les dates d'une facture fournisseur sont décrites). **Le PDF aussi**, lu aplati et vers la sortie standard
   seulement : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. `docs/manual/{de,en,it}/` : confirme qu'il
   n'y a aucun `.tex`.
6. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Issue Tracking Rule*, *Propagation post-patch*, *Recompter ses
   propres comptes rendus*, *Migration breaking policy*) ; `docs/agents-orchestration.md` ;
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La règle et sa preuve**

- **Prémisses** : chaque `fichier:ligne` de la fiche (§ *Ce que le code dit*, 1 à 9) dit-il ce que la fiche lui fait dire (numéros
  de ligne de `910e5d5a`) ? `first_spc` est-il bien le **seul** chemin d'import vers `parse_spc_payload` ? Le parseur tolère-t-il
  CR+LF **sur toutes les lignes lues**, y compris `L_BILLING` et le trailer `EPD` ? Le préfixe `SPC\r\n0200` est-il le seul à
  ajouter, ou la norme/le parseur admet-il d'autres formes (ligne finale sans séparateur, BOM, espaces) qui feraient échouer un
  payload réel pour une autre raison ?
- **Inventaire clos des sites SPC (§ #854)** : refais le `grep` toi-même, plus large (`SPC`, `"\n"`, `'\n'`, `\r`, `lines()`,
  `split`, `starts_with`, `contains("0200")`) dans `crates/*/src/*`, `crates/*/tests/*` et `frontend/src`, **avec une sonde positive**
  (pathspec `crates/*/src/*`, non `crates/*/src`). Un site qui teste ou découpe un payload SPC par `\n` hors du parseur, et que
  le tableau omet ? Chaque ligne « hors classe » l'est-elle vraiment (lis le code) ? Le chemin MCP / les envois / la sauvegarde
  touchent-ils un payload SPC ?
- **AC-1 à AC-5** : chaque test rougit-il avant correctif (T1, T2, T4, T8) et vert avant/après (T3, T5-T7), comme annoncé ? L'encodage
  d'un `\r` dans le QR de `rxing` (mode octet) est-il plausible, ou T2 peut-il échouer pour une autre cause (la fiche
  le prévoit) ? Le banc `jouer` / `Faux` accepte-t-il l'ajout de `SpcCrlf` ? Les négatifs d'AC-2 (e) atteignent-ils bien le préfiltre ?
  AC-3 : `structured_payload()` existe-t-il, produit-il un payload à 31 lignes, et les lignes 32-34 s'y ajoutent-elles comme
  dans `tolerates_31_32_and_34_lines` ? La référence QRR du second cas est-elle réalisable (checksum) ?
- **Les mutations M1 à M5** : chacune rougit-elle ce que la fiche dit, et seulement cela ? M3 (retirer `trim_end_matches('\r')`)
  rougit-elle T2 pour la raison annoncée (IBAN) ? Une régression probable est-elle sans mutation ?
- **#866 / #871 (frontend)** : refais l'inventaire des affichages de quantité (`grep -rn quantity frontend/src`) et des dates ISO
  affichées sur les écrans fournisseur et lots (`invoiceDate`, `dueDate`, `requestedExecutionDate`, `confirmedAt`, `paidAt`, `date`,
  `createdAt`…) : un site manque-t-il (liste des relances fournisseur, écriture d'achat, aperçu du document, export) ? Les types et
  formats de ces champs (`NaiveDate`, `NaiveDateTime`) rendent-ils `formatSwissDate(x.slice(0, 10))` correct ? **La prémisse de
  #871** : relis `supplier-invoices/[id]/+page.svelte` (`cancelSettlement`, `:215-250`) et le test `:202-205` — la fiche a-t-elle
  raison de dire que la date y est déjà au format suisse ? Y a-t-il une autre « confirmation d'annulation de règlement » (client,
  lot, rapprochement) qui affiche une date ISO ? AC-9 (essai par exécution) est-il réalisable tel que décrit ?
- **Périmètre** : un défaut voisin non mécanique est-il intégré à tort ? Un défaut mécanique est-il renvoyé à tort ? Les deux
  issues demandées (C-v0171-7, C-v0171-10) sont-elles justifiées ? Un point écarté sans issue (« on trace toujours ») ?
  Le choix de C-v0171-12 (pas de relecture automatique) tient-il ?
- **Les règles du dépôt** : aucune migration (`ls crates/kesh-db/migrations | wc -l` inchangé : 86 ; pas de P5, P6, P7), aucune
  donnée réelle (ni IBAN, ni émetteur, ni montant d'une pièce du Project Lead dans la fiche, le registre, ni les noms de test),
  aucun mot-clé de fermeture dans les commits prescrits, gate complet au dernier commit de code, E2E complet, captures en thème clair.

**Les textes et ce qu'ils affirment**

- **Le manuel** — axe obligatoire : la phrase prescrite est-elle **vraie au code** ? L'inventaire du § *Manuel* est-il clos : cherche
  dans le `.tex` **et le PDF aplati** toute phrase qui attribue le « Aucune QR-facture exploitable » aux seules résolution/qualité
  du scan, ou qui devient fausse ou muette après la 0.17.1 ? Le geste de relecture décrit (*Télécharger*, *Retirer de la liste*,
  redéposer, *Importer le dossier*) est-il bien celui que le code permet pour un document `qr: none` ? Le manuel dit-il quoi que ce
  soit de la quantité ou des dates d'une facture fournisseur qui deviendrait faux ?
- **CHANGELOG** : les trois puces sont-elles vraies, bien rangées (*Corrigé*), sans fermeture de ticket ? L'intro « aucune migration,
  retour à la 0.17.0 possible sans restauration » est-elle vraie (aucune écriture nouvelle, `kesh_version_min_required` inchangé,
  P2-bis : les versions Cargo ne sont **pas** de la story) ? La puce de #871 est-elle honnête vis-à-vis de AC-9 ?
- **Les décomptes** (sept sites de dates corrigés, dix au grep de contrôle, trois fiches de quantité, douze choix, numéros de ligne) :
  recompte-les depuis la source (`grep -c`, `sed -n`).

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0171-validate-p1.md`. Il contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu (`fichier:ligne` de la
   fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) qui fonde l'affirmation, et le remède
   proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec, pour chaque axe exercé, ce que tu as lu ou cherché. Un « 0 finding »
   sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`, `tee`) — pas même vers un
  fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` et `gh api` en `GET` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`,
  ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- **Jamais `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun autre script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff` (en lecture),
  `pdftotext … -` vers la sortie standard, `gh issue view`, `gh api` en `GET`.
