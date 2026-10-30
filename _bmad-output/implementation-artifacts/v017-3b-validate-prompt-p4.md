# Prompt — validation P4 (ciblée) de la fiche v017-3b (#687 : une facture fournisseur au centime), une lentille, Opus

Tu valides la **remédiation de la passe P3** de la fiche de la Story v017-3b de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11). Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur le
**seul diff** du commit `docs(v017-3b) : remédiation de la validation P3 (refs #687)`. Modèle : **Opus**. **Contexte
frais** : ne t'appuie sur aucun rapport antérieur pour juger, ni sur ce que la fiche dit d'elle-même. Langue du rapport :
**français**.

Worktree `/home/gcorbaz/devel/kesh-0173`, branche `story/v017-3-rapprochement-attente`, partie de `c5101b07`.
Historique : P1 (Sonnet) 0/0/11/14 ; P2 (Opus) 0/0/11/17, suivie d'une **simplification** (C-v017-3-24 : la matrice de
conduite est retirée) ; P3 (Sonnet) 0/0/1/18. À la P3, l'orchestrateur a décidé **C-v017-3-26** : l'avertissement de la
fiche ne s'affiche pas pour une facture annulée, et son texte devient une condition vraie dans tous les autres états.

## À lire

1. Le diff ciblé : `git log --oneline c5101b07..HEAD`, puis `git show <commit de la remédiation P3>` (fiche
   `v017-3b-facture-fournisseur-au-centime.md` et registre `v017-3-choix-autonomes.md`).
2. La fiche entière **après** le diff, pour juger chaque ajout dans son contexte (un ajout peut contredire une phrase
   restée en place).
3. Le code que le diff cite : `crates/kesh-core/src/accounting/mod.rs` (`:22`) ; `crates/kesh-api/src/routes/reconciliation.rs`
   (`:1240-1250`) ; `crates/kesh-db/src/repositories/reconciliation.rs` (`:930-940`) ;
   `crates/kesh-api/src/routes/supplier_invoices.rs` (`:468-500`) ; `crates/kesh-api/src/errors.rs` (le bras
   `AppError::Database`) ; `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte` (`:380-390`, `:495-505`,
   `:615-630`) ; `frontend/src/routes/(app)/supplier-invoices/+page.svelte:93` ;
   `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte` (`:150-165`) ;
   `frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte` (`:195-225`) et `supplier-invoice-form.ts`
   (`:150-235`) ; `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts` (`:1-60`, `:175-205`) ;
   `frontend/src/lib/features/invoices/invoice-helpers.ts` (`:70-90`) ; `docs/api-external.md` (`:725-745`) ;
   `docs/manual/fr/user-manual.tex:2450`.
4. Les rapports P3, **après** ton examen, pour vérifier que chaque finding annoncé appliqué l'est :
   `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p3-R.md` et `-F.md`.

## La lentille — la remédiation P3 et ce qu'elle a pu introduire

- **C-v017-3-26** : le texte de l'avertissement est-il vrai pour chaque état où il s'affiche (ouverte, payée par virement,
  payée par compte interne, payée par un lot) ? Est-il bien absent pour une annulée, partout — AC-10, Vitest, M7, M17,
  captures (AC-14), manuel (voie 1 et voie 3) ? Une phrase **restée en place** (l'ancien texte de l'AC-10, la ligne D8,
  la voie 1, le registre C-v017-3-24) contredit-elle encore la décision sans être marquée « remplacée » ?
- **Les nouvelles prescriptions** : `pub use` de `purchase_line_total` (le chemin et la visibilité tiennent-ils au code ?) ;
  conversion unique par `DbError` (la route de complétion peut-elle vraiment la subir par `?` à l'endroit de l'AC-6 ?) ;
  ordre « bornes avant le `match` » et M15 (le cas « quantité `1.00001` et IBAN invalide » rend-il le code annoncé, sous
  le code et sous la mutation ?) ; lignes de la fiche aux quatre décimales et sélecteur de document laissé (la
  justification est-elle vraie ?) ; véhicule du montage SQL ; clé de `describeSubmitError` et écart **+4** de
  `CLES_RELEVEES` (recompte depuis la garde, qui lit les **sources** : les quatre clés sont-elles toutes appelées par
  `i18nMsg` avec un préfixe de `PREFIXES` ?) ; message aligné (les trois textes portent-ils la même formule ?) ;
  emplacement dans `api-external.md` ; E2E « fournisseur sélectionné d'abord » ; composant testé seul ; CHANGELOG
  (« renomme », changement de contrat).
- **R1** : la règle d'affectation est-elle bien masquée **seulement** par une facture qui porte un indice
  (`routes/reconciliation.rs:1243-1244`, et le cas de la liste pleine) ? L1 et la voie 1 le disent-elles exactement ?
- **Les amendements du registre** : chacun est-il placé **sous** l'entrée qu'il amende (non sous la suivante), daté, et
  sans réécrire l'entrée d'origine ?
- **La propagation** : le symptôme corrigé (« automatiquement », « annulée comprise », « crée la section », « tableau des
  codes », « par exemple quantité 1 ») a-t-il été grepé dans la fiche, le registre et les prompts ? Cite toute occurrence
  restante non marquée comme historique.
- **L'issue demandée (R9)** : le constat est-il exact au code (`balance.ts:23`, `SupplierInvoiceForm.svelte:218-219`) ?

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v017-3b-validate-p4.md`. Il
contient :

1. **Les findings**, numérotés `C1`, `C2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de
   présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable ». Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime
  pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh-0162` ni dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
