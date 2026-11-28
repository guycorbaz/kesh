# Story 63-1 — prompt de la revue de code P5 (Sonnet, passe COMPLÈTE, contexte frais)

Tu conduis la **passe P5** de la story 63-1 de Kesh, **complète** (non ciblée), avec le modèle **Sonnet** (P1 Opus, P2 Sonnet, P3 Haiku ciblée, P4 Opus). Contexte : Kesh est un
logiciel de comptabilité suisse, tenu en production par Guy ; la story corrige trois P1 (#1752, #1753, #1754) : les annulations depuis les fiches contre-passaient
à la date du jour, elles contre-passent désormais à la date de l'écriture d'origine. Worktree `kesh-63-1` (chemin donné par l'orchestrateur), branche
`story/63-1-annulations-datees-origine`. Commit(s) à relire : la remédiation de M1 de la P4, contre `f9a5070f5` (tête de la P4) : commit de code `f5e29384e`, puis le commit de la fiche (l'orchestrateur te donne la tête).
Un diff unique aplati : `git diff f9a5070f5..<tête>`. La remédiation a aussi changé la spec E2E `annulations-datees-origine.spec.ts` (recette du solde en frais bancaires, non rejouée),
`i18n-keys.test.ts` (`sitesTotal` 3979 → 3980) et la clé neuve `invoices-write-off-no-vat-cancel-confirm` ; le L2 (contrepartie sur le compte d'impôt préalable) est un angle mort écrit au § 5 de l'inventaire.

## Pourquoi une passe complète

Quatre passes de suite ont trouvé un **texte faux sur ce qu'une annulation change à la TVA** (M1 de la P1, M1′ de la P2, M2 de la P3, M1 de la P4). La P4
(`$GATE_LOGS/e63/63-1-revue-p4.md`) a montré que l'inventaire clos (`$GATE_LOGS/e63/inventaire-tva-annulation.md`) confondait le **solde** et ses natures : seuls
l'escompte et la perte corrigent la TVA (`crates/kesh-db/src/entities/invoice_settlement.rs`, `corrects_vat` : `Discount | BadDebt`) ; un solde en **frais bancaires**
ou en **reste d'arrondi** n'écrit aucune ligne de TVA (`invoice_settlements_write.rs`, `if nature.corrects_vat() { … } else { Vec::new() }`), et son annulation ne change
aucun décompte. La remédiation touche plusieurs modules (`kesh-i18n`, frontend `invoices`, manuel, doc-comment de `kesh-report`) et change un état visible (la
confirmation d'annulation d'un solde dépend de sa nature) : passe complète.

## La règle, à vérifier toi-même au code (ne la crois pas)

- Changent un décompte de TVA : l'annulation d'un **achat** (facture fournisseur validée, impôt préalable) et celle d'un **solde en escompte ou en perte** (TVA due, ventilation `write_off_vat`).
- Ne changent que des soldes : règlement client, paiement fournisseur, dé-rapprochement de l'un d'eux ou d'un règlement groupé, **solde en frais bancaires ou en reste d'arrondi**.
- Réserve : le dé-rapprochement d'une écriture propre change la TVA récupérable si, et seulement si, elle porte le compte d'impôt préalable.
- Sources : `vat_report.rs` (`generate`, `write_off_rows`, `due_account_debit_write_off_scope`, `recoverable_balance`) ; `invoice_settlements.rs` (`write_off_journal_lines`) ;
  `invoice_settlements_write.rs` (`write_off_invoice`, étape 5) ; `invoice_settlement.rs` (`corrects_vat`) ; `frontend/src/lib/features/invoices/invoices.types.ts` (`writeOffNature`).

## Ce que tu vérifies, comme des findings

1. **M1 de la P4 est fermé partout** : `invoices-write-off-cancel-confirm` (ou la clé neuve) pour les quatre natures, quatre locales ; le repli Svelte de
   `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (choix du texte sur `writeOffNature`, y compris `null`) ; `admin-manual.tex` § *Conformité OLICo Art. 9* (callout du verrou,
   « Pour une facture fournisseur ou un solde ») ; `user-manual.tex` § solde (« TVA comprise », vers la ligne 1972) ; doc-comment de `vat_report.rs` (`write_off_rows`, phrase de règle).
   **Greppe la valeur** sur tout le dépôt : `TVA comprise`, `MWST inbegriffen`, `IVA compresa`, `VAT included`, `ou un solde`, `celle d'un **solde**`, et toute phrase qui dit qu'annuler
   « un solde » change la TVA sans nommer l'escompte ou la perte.
2. **L'inventaire mis à jour** (règle du § 1, sites F2, S1, A1, C1, U6, et les sites ajoutés par la P4) : vérifie-le **comme un finding**, en refaisant la recherche par le symptôme avec
   d'autres mots que les siens. Une assertion dont les deux côtés sortent du même extracteur ne prouve rien.
3. **Le test neuf** (un solde `bank_fees` et un `rounding` montrent le texte sans TVA) : rejoue-le **sous mutation** (condition sur la nature retirée → rouge ; **touche le fichier après
   restauration**). Le test de l'escompte affirme-t-il toujours « TVA comprise » ? Un solde `bad_debt` est-il couvert ?
4. **Quatre locales** : clé(s) neuve(s) présente(s) une fois par locale ; replis Svelte comparés **mot pour mot** au FTL fr-CH (`git grep -nF`) ; DE, IT, EN lus comme langue ;
   compteurs de garde (`sitesTotal` de `i18n-keys.test.ts`) **recomptés depuis la source** si une clé est ajoutée.
5. **Le manuel, PDF compris** : recompile dans une **copie** de `docs/manual` placée sous `/tmp/claude-1000/` (jamais dans le worktree), après suppression des PDF de la copie :
   `make fr/user-manual.pdf fr/admin-manual.pdf` ; aplatis `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` ; **greppe sans apostrophe** (elle est typographique dans le PDF).
   Manuels DE, IT, EN : inexistants (`README.md` seuls), à confirmer.
6. **Rien d'autre n'a bougé** : `git diff --stat f9a5070f5..<tête>` ne nomme que les sites de M1, leurs tests, l'inventaire et la fiche ; **aucun PDF commité**. Les phrases justes
   (achat, escompte et perte, règlement, paiement, dé-rapprochement) ne changent pas.
7. **Les LOW de la P4** (L1 : réserve du manuel d'administration plus étroite que le genre `Entry` ; L2 : contrepartie interne quelconque ; L3 : le dialogue ne montre pas les lignes ;
   L4 : commentaire de test « revue P4 ») : dis lesquels la remédiation a traités, et si le traitement est juste.
8. **Axes non exercés de la P4, à reprendre** : `marketing-brochure.tex` lue en entier ; captures de la recette visuelle des confirmations touchées (règlement, solde par nature,
   dé-rapprochement d'un règlement et d'une écriture propre), si la remédiation en livre ; « formulaires de l'AFC » (aucune source : angle mort, ne le tranche pas).

## Interdits

Tu n'écris **aucun fichier** du dépôt : pas d'`Edit`, d'`Write`, de `git commit`, de `git push`, de `sed -i`, de `scripts/prepare-release.sh`, ni de tout script qui écrit ; pas de `gh` en écriture ; pas de
`docker compose` ni de redémarrage du conteneur ; jamais `GRANT` ni `REVOKE` ; jamais `pkill`, `killall` ni `kill` par motif. Rien sous `~/kesh/`. Les mutations se font dans une **copie** du worktree sous
`/tmp/claude-1000/`, ou se restaurent exactement (`git diff` vide ensuite, puis `touch`). Tes seuls fichiers écrits : ton rapport `$GATE_LOGS/e63/63-1-revue-p5.md`, tes copies sous `/tmp/claude-1000/`.
Les gates lourds : `flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock scripts/mem-guard.sh …`, base `kesh_e631` ; Vitest et toute spec E2E seule sous `$GATE_LOGS/.gate-front.lock`.
Une spec E2E se rejoue contre un backend lancé sur une base jetable `kesh_e631_p5`, port libre vérifié, arrêté par son PID réel ; pas d'E2E complet.

## Rendu

Chaque finding : CRITICAL, HIGH, MEDIUM ou LOW, avec fichier:ligne, la **prémisse lue au code**, et le correctif proposé. Puis :

- **Décompte** par sévérité et **tendance** (P1 0/0/1/13, P2 0/0/1/4, P3 0/0/1/4, P4 0/0/1/4) ;
- **Verdict sur l'inventaire** : sites retrouvés, sites ajoutés, verdicts contestés ;
- la liste des **axes exercés et non exercés** (un « 0 finding » sans cette liste ne compte pas comme passe) ;
- si une passe de plus est requise : une MEDIUM ou plus appelle une P6 (Opus ; complète si la remédiation touche plusieurs modules). **Dès la sixième passe, l'orchestrateur ne remédie plus lui-même.**
