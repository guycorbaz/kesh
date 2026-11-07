# Story 33-12b : le reçu et la partie paiement impriment l'adresse entière (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, ajoutée après le kickoff : P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix »). **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Découpée à la P2 de validation (F-H1 : prémisse fausse, remédiation et correctif dans le même fichier). Rapports : `kesh-gate-logs/e3312-validate-p{1,2}.md`. Commits en `refs #941` ; à la PR : `closes #941` une fois les deux réunies.*

## Story

En tant que débiteur qui lit ou archive une QR-facture, je veux trouver sur le papier le NPA et la localité du créancier et les miens, et la référence au reçu.

## Ce que le code dit (lu)

- La génération est toujours de type S (`invoice_pdf_service.rs:294,311`), où `line1` est la rue et `line2` le numéro. `pdf.rs:765-769, 806-810, 867-871, 935-939` n'imprime que `name`, `line1`, `line2` : **ni NPA ni localité, et la rue séparée du numéro**, au créancier et au débiteur, sur le reçu et la partie paiement. Le code QR les porte. Aucun test n'en dit rien : les jeux d'essai PDF sont de type K (`pdf.rs:1293,1302`, `golden_test.rs:27,36`), et le dépôt ne mesure que des tailles (`pdf.rs:1386-1388`).
- `docs/six-references/ig-qr-bill-v2.4-en.pdf` : **§ 3.6.2** (reçu), « Because of the limited space, it is permitted to […] omit the street name and building number » ; code pays pour un créancier « outside Switzerland, including Liechtenstein », et pour un débiteur hors de Suisse ; le nom peut tenir sur deux lignes. La permission d'omettre rue et numéro n'est qu'au § 3.6.2 ; minimum 6 pt en partie paiement. Ce paragraphe exige une rubrique **Référence** au reçu : `draw_receipt` n'en a aucune (seul `:876`, en partie paiement).
- Dépend de **33-12a** : les lignes sortent de `texte_emis`.

## Critères d'acceptation

1. **AC1** — `lignes_adresse` (posée par la 33-12a) rend, pour le type S, `rue numéro` puis `NPA localité` ; un champ absent, **après repli**, ne laisse ni ligne vide ni espace de tête ; type K inchangé. Tests sur cette fonction, et sur le contenu d'un PDF réel par `printpdf::lopdf` (`extract_text`), non par `pdftotext`, absent de la CI.
2. **AC2** — Pays : § 3.5.4 et § 3.6.2 disent « outside Switzerland, including Liechtenstein » : **pays différent de `CH` → code imprimé**, `LI` compris, créancier et débiteur. Forme `XX-NPA localité` : aucun échantillon du dépôt ne l'établit (treize lus sur dix-huit, tous suisses ; la Style Guide n'est pas au dépôt) : lis les cinq restants, sinon **choix noté au registre**.
3. **AC3** — Largeur, critère décidable : une table littérale des largeurs Helvetica et Helvetica-Bold pour WinAnsi, recopiée dans le code avec sa source (Adobe Core 14 AFM) et contrôlée sur trois valeurs connues. Aucune ligne, à la taille employée, ne dépasse sa colonne : reçu 52 mm ; partie paiement, **par bloc**, créancier à droite du QR, débiteur depuis x = 67 (`pdf.rs`, valeurs à relever). Passé la largeur : coupe à une frontière de mot ; **un mot plus long que la colonne se coupe à la largeur**, sans troncature ni perte.
4. **AC4** — Hauteur : les `y` deviennent **dynamiques** dans chaque colonne, comptés sur le nombre réel de lignes. Borne : au reçu, le bloc du montant et le point de dépôt ; en partie paiement, le bas de page pour le débiteur. Pire cas : `name` 70, `line1` 70, `line2` 16, `postal_code` 16, `town` 35, deux adresses, une référence, au test, calcul écrit. Si le pire cas ne tient pas : au reçu, omets rue et numéro, ce que § 3.6.2 permet ; en partie paiement, où § 3.5.4 ne l'admet pas, réduis la taille, 6 pt au moins, ce que § 3.6.2 évoque : relis la lettre et cite-la.
5. **AC5** — **Référence au reçu** : rubrique et valeur, à leur place selon § 3.6.2, groupée comme en partie paiement, omise sans référence ; clé existante `invoice-pdf-qr-reference`, quatre locales inchangées.
6. **AC6** — Captures regardées : PDF réel, adresse suisse courte, adresse à la borne, créancier `LI`, débiteur étranger, sans référence.
7. **AC7** — Manuel utilisateur `.tex` et PDF, CHANGELOG. Un PDF **déjà figé ne change pas** ; une facture jamais ouverte, ou un rappel régénéré, prendra le nouveau rendu : à écrire tel quel.

La mise en page qui diverge des échantillons SIX (débiteur dans la colonne de droite, ordre compte, référence, débiteur) est tracée à #944 ; le message de la partie paiement non borné en largeur aussi.

Hors de cette story : ciseaux et ligne de séparation (§ 3.7), #944 ; `kesh-report`, #943.

## Mutations (à jouer et consigner)

M1 omettre le NPA au reçu ; M2 omettre la localité en partie paiement ; M3 ligne vide sans numéro ; M4 espace de tête sans rue ; M5 inverser NPA et localité ; M6 omettre le pays d'un débiteur étranger ; M6b omettre `LI-` ; M7 imprimer `CH-` pour un créancier suisse ; M8 tronquer au lieu de couper ; M9 omettre la référence au reçu ; M10 imprimer le débiteur à la place du créancier ; M11 `y` fixes ; M12 mot long perdu.

## Gates

`test-fast.sh --ci`, `cargo build --workspace --all-targets`, **E2E complet**.

## Change Log

- 2026-11-07 — Découpée de la fiche 33-12 à la P2 de validation, F-M2, F-M3, F-M4 remédiés (critère décidable, pays, outil de preuve).
- 2026-11-07 — P3 : P3-M1 (`LI` imprimé), P3-M5 (`y` dynamiques, blocs de la partie paiement), P3-M6 (mot long) remédiés ; lecture du § 3.5.4 corrigée.
