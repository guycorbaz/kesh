# Epic 34 — registre des choix autonomes, clé e3403 (développement de la 34-3, #967)

*Ouvert le 2026-11-10 au développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e3403`, branche
`story/34-3-ecriture-manuelle`, base `7f817792`. Les choix de la fiche sont au registre `e34-choix-autonomes-e342.md`
(C-e342-1 à C-e342-5) ; ceux-ci sont neufs. Mise à jour par union, jamais par écrasement.*

### C-e3403-1 — L'avis d'*Éclater* dans une rangée de tableau sous la ligne, non dans la cellule du compte

- **Contexte** : AC-4 veut l'avis « sous la ligne concernée » ; la cellule du compte est étroite, le texte fait trois
  lignes.
- **Retenu** : une rangée `<tr>` conditionnelle juste sous la ligne, une cellule sur toute la largeur
  (`colspan` 4 ou 5 selon la colonne des projets), portant `data-line`.
- **Écartées** : l'avis dans la cellule du compte (illisible à cette largeur) ; un avis unique sous le tableau (ne dit pas
  quelle ligne).
- **Réversible** : oui, gabarit seul.

### C-e3403-2 — Un composant d'avis partagé, `CounterpartyClaimNotice.svelte`

- **Contexte** : les deux dialogues portent le même avis (AC-4) ; la règle « rôle Receivable ou Payable » est pure.
- **Retenu** : `claimRoleOf` dans `reconciliation-counterparties.ts` (avec `counterpartyAccounts`), et un composant
  d'affichage partagé qui lit le compte choisi dans la liste proposée. `role="note"`, teinte ambre (déjà employée par le
  badge *Facture fournisseur* de la même page).
- **Écartée** : copier le gabarit dans chaque dialogue (DRY).
- **Réversible** : oui.

### C-e3403-3 — La garde des replis porte sur treize clés, non neuf

- **Contexte** : l'AC-6 nomme une liste fermée de neuf clés (huit + le bouton).
- **Retenu** : `ecriture-manuelle-i18n.test.ts` y ajoute le titre, la validation et les deux avis (AC-1, AC-4) : treize
  clés, toutes asserties relevées, repli égal au catalogue fr-CH, présentes dans les quatre locales ; les huit qui citent
  le geste nomment le geste neuf dans chaque locale.
- **Réversible** : oui.

### C-e3403-4 — Formes verbales du geste dans les messages

- **Contexte** : l'AC-6 admet « Écriture manuelle » ou « passée par une écriture manuelle » ; les messages disaient
  « affectez la transaction à la main ».
- **Retenu** : fr « passez la transaction par une écriture manuelle », « passées par une écriture manuelle » ; de
  « erfassen Sie die Transaktion als manuelle Buchung », « als manuelle Buchung erfasst » ; en « post the transaction as a
  manual entry », « posted as manual entries » ; it « contabilizzare la transazione con una registrazione manuale »,
  « contabilizzate con una registrazione manuale ». Les citations du bouton prennent le libellé exact (« N’utilisez
  « Écriture manuelle » que si… », « «Manuelle Buchung» », « “Manual entry” », « «Registrazione manuale» »).
  Traductions de/en/it proposées par l'agent, à relire par Guy s'il le souhaite.
- **Réversible** : oui, catalogues.

### C-e3403-5 — Le second côté d'un virement entre vos comptes se rejette (manuel)

- **Contexte** : l'AC-8 dit « le second côté se rattache ou se rejette (`:2991`, #823) » ; `:2991` dit, lui, que
  l'écriture du virement se rattache à l'une des deux transactions et que l'autre « reste sans écriture — rejetez-la ».
  Une écriture ne se rattache qu'à une seule transaction (`user-manual.tex`, « Les conditions d'une écriture »).
- **Retenu** : le § *Écriture manuelle* suit `:2991` : le second côté se **rejette**.
- **Écartée** : « se rattache » (le rattachement serait refusé : l'écriture est déjà rapprochée du premier côté).
- **Réversible** : oui, texte.

### C-e3403-6 — Le repli de `reconciliation-labels-more-capped` aligné sur le catalogue (défaut préexistant)

- **Contexte** : le test neuf de l'AC-6 a rougi d'avance sur cette clé : repli Svelte « rattachées à un règlement »,
  catalogue fr « rattachées à une écriture existante » (dérive laissée par la v017-3a, qui avait renommé le geste au
  catalogue seulement). Aucune garde ne le voyait (un seul site).
- **Retenu** : corrigé ici, dans la zone de la story (limite courte).
- **Réversible** : sans objet.

### C-e3403-7 — Le manuel nomme l'issue #981 pour les comptes de TVA

- **Contexte** : la fiche écrit que l'avis ne couvre pas les comptes de TVA ; l'issue est ouverte (#981).
- **Retenu** : le § *Écriture manuelle* le dit, avec le numéro, comme il cite #823 et #827.
- **Réversible** : oui.

### C-e3403-8 — L'avis créanciers reformulé (décision de l'orchestrateur, revue de code P1, B1-5)

- **Contexte** : le texte de l'AC-4 (« saisissez et validez la facture sans la payer depuis sa fiche ») se lisait aussi
  « validez-la depuis sa fiche » ou « ne la payez pas depuis sa fiche » ; les traductions reprenaient l'ambiguïté.
- **Retenu** (sens dicté par l'orchestrateur) : fr « saisissez la facture et validez-la depuis sa fiche, sans y enregistrer
  de paiement : le paiement est cette transaction, et la facture vous sera proposée ici, à accepter » ; de, en, it de même
  sens ; repli, test et manuel (`user-manual.tex`, § *Écriture manuelle*) alignés. L'AC-4 de la fiche garde le texte
  d'origine (relevé daté) ; ce choix le remplace.
- **Réversible** : oui, catalogues.

### C-e3403-9 — L'avis suit le rôle, non le sens de la transaction (revue de code P1, E1-2)

- **Contexte** : un débit sur le compte débiteurs (remboursement d'un client) ou un crédit sur le compte créanciers reçoit
  le conseil du cas nominal, qui ne s'y applique pas (le texte reste conditionnel : « Pour un paiement de client… »).
- **Retenu** : tel quel dans la story ; une variante selon le signe est demandée en issue.
- **Réversible** : oui.
