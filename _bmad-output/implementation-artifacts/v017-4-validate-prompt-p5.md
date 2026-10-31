# Prompt — validation P5 de la fiche v017-4 (#849), passe ciblée, une lentille, Sonnet

Tu fais la **passe ciblée** de validation de la fiche de la Story v017-4 de Kesh (factures fournisseur : brouillon →
validée → payée ; Rust/Axum, SvelteKit, MariaDB 10.11) et de ses deux sous-fiches. Modèle : **Sonnet**. **Contexte
frais**. Langue du rapport : **français**. Worktree `/home/gcorbaz/devel/kesh-0175`, branche
`story/v017-4-facture-fournisseur-brouillon`.

La validation P4 (Opus, lentilles R et F) a rendu 0 CRITICAL, 0 HIGH, 2 MEDIUM, 10 LOW, tous remédiés par **un seul
commit**, celui dont le message est `docs(v017-4) : remédiation de la validation P4 (refs #849)` :
`git log -1 --format=%H --grep="remédiation de la validation P4"`. Cette remédiation ne touche que la rédaction et le
niveau d'un test, aucune règle d'exécution. *La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire*
(`CLAUDE.md`, § *La passe ciblée*) : ton périmètre est **ce commit**, et ce qu'il affirme du code.

## À lire

1. Le diff : `git show <ce commit>` (fichiers sous `_bmad-output/implementation-artifacts/` : la fiche parente
   `v017-4-facture-fournisseur-brouillon.md`, les sous-fiches `v017-4a-…-backend.md`, `v017-4b-…-ecran.md`, le registre
   `v017-4-choix-autonomes.md`, C-v017-4-41 et 42).
2. Les rapports de la P4 : `/home/gcorbaz/devel/kesh-gate-logs/v0174-validate-p4-R.md` et `…-F.md` (lecture seule).
3. Le code que le diff cite : `crates/kesh-api/src/errors.rs` (≈ `:4560-4600`, `supplier_invoice_cancel_blocked_text`),
   `crates/kesh-api/src/retry.rs` (`retry_app_on_deadlock`), `crates/kesh-db/src/retry.rs`,
   `crates/kesh-api/tests/admin_full_import_e2e.rs` (`strip_column` `:930`, `ouvrir_tout` `:3632`, `recrire_table`
   `:3684`, `rezip_tout`), `crates/kesh-api/tests/audit_route_registry.rs` (en-tête, points (ii) et (vi)),
   `crates/kesh-db/src/repositories/payment_batches.rs` (annulation d'un lot), `crates/kesh-db/src/repositories/imported_supplier_invoices.rs`
   (`attach_in_tx`, `detach_in_tx`), `crates/kesh-db/src/test_fixtures.rs` (`attendre_une_requete_en_cours`),
   `frontend/src/lib/features/supplier-invoices/invoice-cancel.ts`, `SupplierInvoiceForm.svelte`, `docs/api-external.md`
   (`:953`), `crates/kesh-db/test-schema/0001_schema_squash.sql` (clé `fk_supplier_invoices_purchase_je`).
4. `CLAUDE.md` du worktree et `docs/agents-orchestration.md`.

## La lentille unique — la remédiation de la P4

Pour chaque finding de la P4 (R1 à R4, F1 à F8) : le remède est-il dans le diff, **exact au code** (relis la ligne qu'il
cite), complet, et sans défaut neuf — contradiction avec un autre passage de la fiche (greper la **valeur** sur la fiche
entière : « dix-neuf », « vingt-deux », « treize », « douze », `retry_on_deadlock("supplier_invoices::update"`,
`supplier-invoices-cancel-blocked-draft`, « n'est pas ajouté », `:3622`, `:3687`), avec le registre, avec une mutation
(M20-quater, M34), avec le découpage (AC-13 en 4a, AC-25 en 4b) ? En particulier :
- le texte du motif brouillon (AC-13) vaut-il pour un Comptable, une clé `read-write` et un brouillon de L8 ? sa clé
  suit-elle le patron des clés voisines de `errors.rs` ?
- le test piloté au niveau du dépôt tue-t-il M20-quater **quelle que soit la victime** choisie par InnoDB ? le test de
  route est-il réalisable (la chorégraphie avec une route HTTP) ?
- la prop `hasDocument` et le `Pick` de douze champs : le formulaire peut-il les recevoir sans casser la page de
  complétion ?
- **le manuel** : le diff touche-t-il un texte que l'AC-29 ou l'AC-20 doit reprendre au manuel (L8 et son chemin, le
  scan caché, le motif d'annulation d'un brouillon) ? Contrôle au `.tex` et au PDF aplati
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, sortie standard seulement).

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0174-validate-p5.md` :

1. **Les findings**, numérotés `P1`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu, **la ligne
   lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé.
2. **Un tableau** finding P4 → remède vérifié (oui / non / régression), ligne lue citée.
3. **Les axes exercés** et **les axes non exercés**, un par un. Un « 0 finding » sans cette liste ne compte pas.
4. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`, ni
  aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
