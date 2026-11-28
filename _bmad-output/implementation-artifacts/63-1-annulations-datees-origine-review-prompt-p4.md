# Story 63-1 — prompt de la revue de code P4 (Opus, passe COMPLÈTE, contexte frais)

Tu conduis la **passe P4** de la story 63-1 de Kesh, **complète** (non ciblée), avec le modèle **Opus** (P1 Opus, P2 Sonnet, P3 Haiku ciblée). Contexte : Kesh est un
logiciel de comptabilité suisse, tenu en production par Guy ; la story corrige trois P1 (#1752, #1753, #1754) : les annulations depuis les fiches contre-passaient
à la date du jour, elles contre-passent désormais à la date de l'écriture d'origine. Worktree `/home/gcorbaz/devel/kesh-63-1`, branche
`story/63-1-annulations-datees-origine`. Commit à relire : `8f2699ceb` (la remédiation de la classe de défauts « ce qu'une annulation change à la TVA »), contre
`22019e19e` (tête de la P3). Un diff unique aplati : `git diff 22019e19e..8f2699ceb`.

## Pourquoi une passe complète

Trois passes de suite ont trouvé un **texte faux sur ce qu'une annulation change à la TVA** (M1 de la P1, M1′ de la P2, M2 de la P3) : chaque remède local a laissé ou créé
un autre site faux. La remédiation a donc été conduite **à partir du symptôme**, par un **inventaire clos** : `$GATE_LOGS/e63/inventaire-tva-annulation.md` (72 sites : 43 justes,
25 faux, 4 ambigus avant ; 72 justes après ; 29 sites modifiés). Elle touche trois modules de production (`kesh-report` : un doc-comment ; `kesh-i18n` : quatre catalogues ;
frontend : trois composants, dont **un changement de comportement** : `CancelReconciliationDialog.svelte` ne montre plus la phrase de TVA que pour une écriture propre) :
elle ne se contente donc pas d'une passe ciblée, et elle change un état visible de l'écran.

## La règle, à vérifier toi-même au code (ne la crois pas)

- **Seuls l'annulation d'un achat** (facture fournisseur validée) **et d'un solde** client (escompte, perte, avec sa ligne de TVA) changent un décompte de TVA.
- Un **règlement client**, un **paiement fournisseur**, et le **dé-rapprochement** de l'un d'eux ou d'un règlement groupé ne changent que des soldes (contrepartie, débiteurs, créanciers) :
  aucune ligne de TVA. La contrepartie peut être un compte interne, non toujours la banque.
- **Réserve** (posée par la remédiation) : le dé-rapprochement d'une **écriture propre** (éclatement, règle, rapprochement manuel, écriture saisie au journal) change la TVA récupérable
  si, et seulement si, l'écriture porte le compte d'impôt préalable.
- Sources : `crates/kesh-report/src/vat_report.rs` (`generate`, `write_off_rows`, `recoverable_balance`) ; `crates/kesh-db/src/repositories/invoice_settlements.rs`
  (`settlement_journal_lines`, `write_off_journal_lines`) ; `supplier_invoices.rs` (`generate_purchase_journal_lines`, `entry_lines` du paiement) ;
  `reconciliation_cancel.rs` (`ReconciliationKind`, `classify`) ; `journal_entries.rs` (`reversal_blocker`). **Conteste la réserve** si tu trouves un chemin qui la contredit
  (par exemple une écriture de vente ou d'achat dé-rapprochée et contre-passée, ou un compte de TVA due lu par le décompte depuis le grand livre).

## Ce que tu vérifies, comme des findings

1. **L'inventaire est un finding à vérifier, non une preuve.** Une assertion dont les deux côtés sortent du même extracteur est vraie par construction : **refais la recherche toi-même, en partant du
   symptôme** (toute phrase qui dit ce qu'une annulation, une contre-passation, un dé-rapprochement ou le verrou change à la TVA, au décompte, à la déclaration), avec d'autres mots que ceux de
   l'inventaire (`rapport TVA`, `déposé`, `impôt préalable`, `solde du compte`, `remboursement`, `trimestre`, `semestre`, `période close`, `OLICo`, `fiduciaire`, et leurs équivalents DE, IT, EN),
   et **reprends les sites que l'inventaire déclare exclus** (§ 2 : brochure `marketing-brochure.tex`, motifs de verrou sans mot de TVA, `CHANGELOG.md`) et les **angles morts** (§ 5). Chaque site
   que tu trouves et que l'inventaire n'a pas est un finding ; chaque verdict « juste » que tu contestes aussi.
2. **Les phrases justes n'ont pas bougé** : achat, solde, verrou appliqué à l'achat. `git diff 22019e19e..8f2699ceb` ne doit contenir aucune de leurs phrases.
3. **Aucune règle de code changée.** Seule exception de comportement : la condition de `vatText` (`view.kind === 'entry' || view.kind == null`) ; vérifie qu'elle est juste pour **chaque** genre de
   `ReconciliationTransactionResponse.kind` (`reconciliation.types.ts:303`), y compris `null`, et que `unlinkOnly` reste exclu.
4. **Quatre locales** : les six clés de l'inventaire (§ 3.3), `fr-CH`, `de-CH`, `it-CH`, `en-CH` ; les replis Svelte comparés **mot pour mot** au FTL fr-CH (`git grep -nF`) ; DE, IT, EN lus en
   tant que langue (registre, sens : « la TVA non », « die MWST nicht », « l'IVA no », « the VAT does not »). Les clés `-vat` ne changent pas de nom ; aucun compteur de garde (`sitesTotal`) ne
   bouge — vérifie-le.
5. **Le manuel, PDF compris** : recompile dans une **copie** de `docs/manual` placée sous `/tmp/claude-1000/` (jamais dans le worktree) : `make fr/user-manual.pdf fr/admin-manual.pdf` ; aplatis
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` ; **greppe sans apostrophe** (elle est typographique dans le PDF : `l’AFC`, `d’impôt`). Lis en entier : `admin-manual.tex` § *Conformité OLICo Art. 9*
   (puce du verrou), § *La date du jour est celle de Zurich* ; `user-manual.tex` § *Arrêter les livres à une date*, § *Enregistrer et annuler un règlement*, § règlement fournisseur,
   § *Annuler une facture fournisseur*, § *Annuler un rapprochement*, § décompte de TVA (« Le solde annulé »). Vérifie aussi le renvoi `\S\ref{sec:verrou-periode}` ajouté (cible, numéro dans le PDF).
   Les manuels DE, IT, EN n'existent pas (`docs/manual/{de,it,en}/README.md` seuls) : confirme-le.
6. **Tests** : le test neuf `CancelReconciliationDialog.test.ts` (« aucune phrase de TVA pour un règlement… ») prouve-t-il quelque chose ? Rejoue-le **avec une mutation** (rétablis la condition
   `view && !view.unlinkOnly` seule : il doit rougir ; **touche le fichier après restauration**, mémoire « mutation restaurée, binaire périmé »). Les assertions réécrites (T2, T4, T5, T6 de l'inventaire)
   affirment-elles le texte neuf, et non l'absence de l'ancien ?
7. **Doc-comment de `vat_report.rs:79-90`** : vrai pour un solde annulé depuis et avant la 0.21.0 ? Le « sans TVA » ajouté pour le règlement, le paiement et le dé-rapprochement est-il exact ?
8. **Axes non exercés des passes précédentes, à reprendre** (P3, § « Axes exercés et non exercés ») : manuels DE, IT, EN (phrases « ne change aucun décompte » et « Le trimestre déjà déclaré ») ;
   « formulaires de l'AFC » (aucune source dans le dépôt : signale-le comme angle mort, ne le tranche pas) ; gate complet et E2E complet (hors périmètre d'une revue, mais dis si la spec E2E
   `reconciliation-cancel.spec.ts`, dont l'assertion a changé, est cohérente avec le composant : lis-la).
9. Rien d'autre n'a bougé : `git diff --stat 22019e19e..8f2699ceb` ne nomme que les fichiers de l'inventaire et la fiche ; **aucun PDF commité**.

## Interdits

Tu n'écris **aucun fichier** du dépôt : pas d'`Edit`, d'`Write`, de `git commit`, de `git push`, de `sed -i`, de `scripts/prepare-release.sh`, ni de tout script qui écrit ; pas de `gh` en écriture ; pas de
`docker compose` ni de redémarrage du conteneur ; jamais `GRANT` ni `REVOKE` ; jamais `pkill`, `killall` ni `kill` par motif. Rien sous `~/kesh/`. Les mutations se font dans une **copie** du worktree sous
`/tmp/claude-1000/`, ou se restaurent exactement (`git diff` vide ensuite). Tes seuls fichiers écrits : ton rapport `$GATE_LOGS/e63/63-1-revue-p4.md`, tes copies sous `/tmp/claude-1000/`.
Les gates lourds : `flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock scripts/mem-guard.sh …`, base `kesh_e631` ; Vitest sous `$GATE_LOGS/.gate-front.lock`.

## Rendu

Chaque finding : CRITICAL, HIGH, MEDIUM ou LOW, avec fichier:ligne, la **prémisse lue au code**, et le correctif proposé. Puis :

- **Décompte** par sévérité et **tendance** (P1 0/0/1/13, P2 0/0/1/4, P3 0/0/1/4) ;
- **Verdict sur l'inventaire** : sites que tu as retrouvés, sites que tu as ajoutés, verdicts contestés ;
- la liste des **axes exercés et non exercés** (un « 0 finding » sans cette liste ne compte pas comme passe) ;
- si une passe de plus est requise : une MEDIUM ou plus appelle une P5 (Sonnet, complète si la remédiation touche plusieurs modules).
