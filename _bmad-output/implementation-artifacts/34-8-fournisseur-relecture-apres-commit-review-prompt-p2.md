# Revue de code P2 — Story 34-8 (#938) : une facture fournisseur n'est plus annoncée en échec après son commit

Revue adversariale **complète**, confiée à **Opus**, trois lentilles en contexte frais : **B** (Blind Hunter), **E** (Edge Case
Hunter), **A** (Acceptance Auditor). La passe P1 (Sonnet) a rendu 2 MEDIUM et 7 LOW, remédiés aux commits `c7251f56` et
`69855016` ;
le motif connu est que la sévérité se déplace vers ce qu'on vient d'écrire : **relisez d'abord la remédiation**, puis la
story entière.

## Périmètre

- Worktree : `/home/gcorbaz/devel/kesh-e348`, branche `story/34-8-fournisseur-relecture-apres-commit`.
- Diff complet : `git -C /home/gcorbaz/devel/kesh-e348 diff 665de151..HEAD` (base `665de151`).
- Diff de la remédiation P1 : `git -C /home/gcorbaz/devel/kesh-e348 diff 5e6ed701..69855016` (commits `c7251f56` et
  `69855016` : le second corrige le montage du test de B-3, refusé par une contrainte, et la coupure de l'E2E de A-1).
- Fiche : `_bmad-output/implementation-artifacts/34-8-fournisseur-relecture-apres-commit.md` (AC-1 à AC-9, mutations M1 à
  M7, Change Log, Dev Agent Record).
- Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/e348-review-p1-{B,E,A}.md`.
- Choix : `_bmad-output/implementation-artifacts/e34-choix-autonomes-e343.md` (C-e343-9 à 12, 15) et
  `e34-choix-autonomes-e348.md` (C-e348-1 à 6).
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e348/` (rouges, `mut/`, gates `p1b-*`, E2E `e2e-r3/` ; `p1-*` et
  `e2e-r2/` sont ceux de `c7251f56`, avant le second correctif) ; captures dans
  `/home/gcorbaz/devel/kesh-gate-logs/e348/captures/` (`e348-*.png`).

## Interdits

Lecture seule. Aucune commande qui écrit, compile, exécute un test ou lance un script (`cargo`, `npm`, `npx`,
`scripts/*.sh`, `scripts/prepare-release.sh` compris), aucun `git` qui écrit (`checkout`, `commit`, `stash`, `reset`…),
aucune redirection (`>`, `>>`, `tee`) ni fichier temporaire. Permis : `git diff`, `git show`, `git log`, `git grep`,
`/usr/bin/grep`, `sed -n`, `cat`, `pdftotext … -` vers la sortie standard, la lecture des images. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` copiée dans le rapport. Le seul fichier écrit est ton rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e348-review-p2-<B|E|A>.md` selon ta lentille : findings numérotés `<lentille>-n`,
sévérité (CRITICAL, HIGH, MEDIUM, LOW), preuve (fichier:ligne, extrait), correction proposée. Puis **la liste des axes
exercés et celle des axes non exercés** : une passe qui ne la donne pas ne compte pas.

## Axes de la remédiation P1 (à exercer en premier)

1. **`docs/api-external.md`** (B-1) : les sites récrits (tableau des routes de la 0.17.0, paragraphe des champs de lecture,
   `cancel`, complétion, `sourceDocument`, rattachement, détachement). Le paragraphe neuf sur l'instantané dit-il vrai du
   code (`get_settlement_view_in_tx`, C-e343-10) ? La liste des neuf gestes est-elle juste ? « Ils ne se contredisent pas
   entre eux » est-il tenu ? Reste-t-il un « relue » qui décrit une réponse de serveur (`git grep -n relue`) ?
2. **Doc-comments** : `update_supplier_invoice`, les clients `supplier-invoices.api.ts`, `imported-supplier-invoices.api.ts`,
   `imported-supplier-invoices.types.ts`, la fiche Svelte.
3. **`pay()`** (B-2) : le repli retiré ; `payError` peut-il rester vide sur une erreur ? `sitesTotal` 2924 et son commentaire.
4. **Le test neuf `commit_with_view_rolls_back_when_the_view_cannot_be_read`** (B-3) : prouve-t-il ce qu'il dit ? La
   mutation M7 (`mut/M7.log`) le tue-t-elle pour la bonne raison ?
5. **Le doc-comment de `find_by_supplier_invoice_id_scoped`** (B-4) : l'hypothèse sur le démarrage est-elle exacte
   (`main.rs`, `pre_upgrade_backup`) ?
6. **Les tests `attach_returns_what_was_written` et `detach_returns_what_was_written`** (A-3).
7. **Les deux tests E2E neufs** de `frontend/tests/e2e/supplier-invoice-unknown-outcome.spec.ts` (A-1) : la coupure de la
   relecture ne touche-t-elle que le `GET` d'après le paiement ? Le rattachement coupé avant le serveur n'écrit-il rien ?
   Les captures `e348-relecture-en-echec.png` et `e348-selecteur-issue-inconnue.png` montrent-elles l'état, en thème clair ?

## Axes de la story entière

8. **`commit_with_view`** : rollback sur `Err`, sur vue absente et sur lecture en échec ; commit après la lecture ;
   `view_id` sur `Ok` seulement.
9. **Les cinq enveloppes et les quatre `_once`** : plus de `tx.commit()` direct ; formes `_in_tx` inchangées.
10. **Les neuf handlers** et le rejeu sur interblocage (AC-7).
11. **La garde de structure** et sa sonde positive (`common/autour_appel.rs`, `supplier_invoice_response_e2e.rs`).
12. **L'écran** : sept gestes, `cycleOutcome`, deux avis, `attach` (trois branches), `detach`, 4xx inchangé.
13. **Textes** : quatre locales, CHANGELOG, manuel. **Contrôle le PDF** `docs/manual/fr/user-manual.pdf`, aplati :
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "Si la réponse se perd[^.]*\. [^.]*\."`.
14. **Les déclarations** du Dev Agent Record et du Change Log : décomptes recomptés aux deux bornes, gates déclarés tels
    qu'ils ont tourné.

## Lentilles

- **B (Blind Hunter)** : le diff seul, sans la fiche. Défauts de code, de logique, de concurrence, d'erreurs avalées.
- **E (Edge Case Hunter)** : chaque branche et chaque bord — vue `None`, lecture en échec, rollback qui échoue, rejeu,
  réponse perdue puis relecture qui échoue, document joint ailleurs pendant la relecture, double clic, autre société.
- **A (Acceptance Auditor)** : chaque AC contre le code et les tests, les mutations M1 à M7, les findings P1 et leur
  remédiation, les choix, le manuel et son PDF, les captures.
