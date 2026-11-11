# Revue de code P3 — Story 34-8 (#938) : une facture fournisseur n'est plus annoncée en échec après son commit

Revue adversariale **complète**, confiée à **Sonnet**, trois lentilles en contexte frais : **B** (Blind Hunter), **E** (Edge
Case Hunter), **A** (Acceptance Auditor). Tendance : P1 (Sonnet) 2 MEDIUM, 7 LOW ; P2 (Opus) 2 MEDIUM, 9 LOW, remédiés au
commit `3f5d36c4`. La sévérité se déplace vers ce qu'on vient d'écrire : **regard appuyé sur la remédiation P2**, puis la
story entière.

## Périmètre

- Worktree : `/home/gcorbaz/devel/kesh-e348`, branche `story/34-8-fournisseur-relecture-apres-commit`.
- Diff complet : `git -C /home/gcorbaz/devel/kesh-e348 diff 665de151..HEAD` (base `665de151`).
- Diff de la remédiation P2 : `git -C /home/gcorbaz/devel/kesh-e348 diff 421a8f02..HEAD` (code : `3f5d36c4` ; manuel,
  registre et fiche : le commit suivant).
- Fiche : `_bmad-output/implementation-artifacts/34-8-fournisseur-relecture-apres-commit.md` (AC, mutations M1 à M9,
  Change Log, Dev Agent Record).
- Rapports P1 et P2 : `/home/gcorbaz/devel/kesh-gate-logs/e348-review-p{1,2}-{B,E,A}.md`.
- Choix : `_bmad-output/implementation-artifacts/e34-choix-autonomes-e343.md` (C-e343-9 à 12, 15) et
  `e34-choix-autonomes-e348.md` (C-e348-1 à 9).
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e348/` (`mut/` et `SYNTHESE.log`, gates `p2-*`, E2E `e2e-r4/`) ; captures
  `e348-*.png` dans `captures/`.

## Interdits

Lecture seule. Aucune commande qui écrit, compile, exécute un test ou lance un script (`cargo`, `npm`, `npx`,
`scripts/*.sh`, `scripts/prepare-release.sh` compris), aucun `git` qui écrit (`checkout`, `commit`, `stash`, `reset`…),
aucune redirection (`>`, `>>`, `tee`) ni fichier temporaire. Permis : `git diff`, `git show`, `git log`, `git grep`,
`/usr/bin/grep`, `sed -n`, `cat`, `pdftotext … -` vers la sortie standard, la lecture des images. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` copiée dans le rapport. Le seul fichier écrit est ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e348-review-p3-<B|E|A>.md` selon ta lentille : findings numérotés `<lentille>-n`,
sévérité (CRITICAL, HIGH, MEDIUM, LOW), preuve (fichier:ligne, extrait), correction proposée. Puis **la liste des axes
exercés et celle des axes non exercés** : une passe qui ne la donne pas ne compte pas.

## Axes de la remédiation P2 (à exercer en premier)

1. **`reloadAfterStaleRefusal`** et `isStaleState` étendu (`+page.svelte`, E-1, C-e348-7) : chaque geste qui l'emploie
   (payer, annuler le règlement, annuler, valider, repasser en brouillon) ; les codes ajoutés sont-ils ceux que le serveur
   rend pour ces refus (`crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/errors.rs`) ? Un refus qui ne dit **pas** l'écran
   périmé déclenche-t-il une relecture à tort ? Un message de refus reste-t-il affiché sous un bloc qui disparaît, ou
   réapparaît-il plus tard ? L'encadré `reread-failed` revient-il à bon escient ?
2. **`retry: false`** (E-2, C-e348-8) : `apiClient.get`, `getSupplierInvoice` et `reloadAfterRefusal`. Le chargement
   initial garde-t-il son rejeu ? Un autre appelant de `apiClient.get` est-il touché ?
3. **Les replis retirés** (B-2 = A-2) : chaque `catch` touché peut-il laisser un message vide sur une erreur réelle ?
   `sitesTotal` 2921 et ses annotations sont-ils justes ?
4. **Le rattachement** (B-4, C-e348-9) : la comparaison `importedSupplierInvoiceId` ; un document joint ailleurs, autre que
   le sien, garde-t-il l'avertissement ?
5. **Textes** : `docs/api-external.md:609` et `:623`, `supplier-invoices.types.ts`, `unknown-outcome.ts`, la note du module
   `imported_supplier_invoices.rs` (B-3), le manuel (`user-manual.tex`, phrase du second envoi) et **son PDF, aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "Un second envoi[^.]*\."`.
6. **Les tests et mutations neufs** : Vitest « second clic » (trois gestes, relecture encore en échec, B-4, `retry: false`)
   et M8, M9 (`mut/M8.log`, `mut/M9.log`). M8 a été mesurée deux fois : la première sur la remédiation non commitée, que la
   restauration du script de mutation (`git checkout --`) a effacée puis qui a été refaite — vérifie que le code commité
   `3f5d36c4` contient bien tout ce que le Change Log déclare.

## Axes de la story entière

7. `commit_with_view`, les cinq enveloppes, les quatre `_once`, les neuf handlers, le rejeu sur interblocage.
8. La garde de structure et sa sonde positive.
9. L'écran : sept gestes, `cycleOutcome`, deux avis, `attach`, `detach`, 4xx.
10. Les déclarations du Dev Agent Record et du Change Log (décomptes aux deux bornes, gates tels qu'ils ont tourné,
    File List).

## Lentilles

- **B (Blind Hunter)** : le diff seul, sans la fiche.
- **E (Edge Case Hunter)** : chaque branche et chaque bord — refus périmé puis relecture en échec, double clic pendant une
  relecture, document joint ailleurs, autre société, rejeu.
- **A (Acceptance Auditor)** : chaque AC et chaque finding P1 et P2 contre le code, les tests, les mutations, les choix, le
  manuel et son PDF, les captures.
