# Story 33-12b : le reçu et la partie paiement impriment l'adresse entière (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, ajoutée après le kickoff : P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix »). **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Découpée à la P2 de validation (F-H1 : prémisse fausse, remédiation et correctif dans le même fichier). Rapports : `kesh-gate-logs/e3312-validate-p{1,2,3,4}.md`. Le registre se crée au développement. Commits en `refs #941` ; à la PR : `closes #941` une fois les deux réunies.*

## Story

En tant que débiteur qui lit ou archive une QR-facture, je veux trouver sur le papier le NPA et la localité du créancier et les miens, et la référence au reçu.

## Ce que le code dit (lu)

- La génération est toujours de type S (`invoice_pdf_service.rs:294,311`), où `line1` est la rue et `line2` le numéro. `pdf.rs:765-769, 806-810, 867-871, 935-939` n'imprime que `name`, `line1`, `line2` : **ni NPA ni localité, et la rue séparée du numéro**, au créancier et au débiteur, sur le reçu et la partie paiement. Le code QR les porte. Aucun test n'en dit rien : les jeux d'essai PDF sont de type K (`pdf.rs:1293,1302`, `golden_test.rs:27,36`), et le dépôt ne mesure que des tailles (`pdf.rs:1386-1388`).
- `docs/six-references/ig-qr-bill-v2.4-en.pdf` : **§ 3.6.2** (reçu), « Because of the limited space, it is permitted to […] omit the street name and building number » ; code pays pour un créancier « outside Switzerland, including Liechtenstein », et pour un débiteur hors de Suisse ; le nom peut tenir sur deux lignes. La permission d'omettre rue et numéro n'est qu'au § 3.6.2 ; minimum 6 pt en partie paiement. Ce paragraphe exige une rubrique **Référence** au reçu : `draw_receipt` n'en a aucune (seul `:876`, en partie paiement).
- Dépend de **33-12a** : les lignes sortent de `texte_imprime`.

## Critères d'acceptation

*Réduite le 2026-11-07, après quatre passes : les métriques de police, la réduction de taille et le pire cas absolu sortent de la story, tracés à #944. Elle corrige ce qui manque sur une adresse ordinaire, sans prétendre tout borner.*

1. **AC1** — La fonction pure de la 33-12a rend, en type S, `name`, `rue numéro`, `NPA localité` ; un champ vide après `texte_imprime` ne laisse ni ligne vide ni espace de tête. Type K inchangé. Reçu et partie paiement, créancier et débiteur, par ce seul chemin.
2. **AC2** — Pays différent de `CH` → préfixe `XX-` devant le NPA, `LI` compris, créancier comme débiteur, reçu comme partie paiement. Les figures 19 et 20 de la directive montrent `LI - 9490 Vaduz` et `DE- 78462 Konstanz`, en image : forme retenue sans espace, `LI-9490 Vaduz`, notée au registre.
3. **AC3** — Coupe simple : une ligne au-delà de `MAX_CARACTERES` (constante par colonne, relevée au code et justifiée au commentaire par la largeur moyenne prudente d'Helvetica à la taille employée) se coupe à une frontière de mot ; un mot plus long se coupe à la limite. Rien n'est tronqué. Pas de table de métriques.
4. **AC4** — Les `y` déjà relatifs suivent le nombre réel de lignes. Tests : adresse suisse courte, rue longue d'une soixantaine de caractères, débiteur étranger, créancier `LI`, sans numéro ; les positions du cas ordinaire ne descendent pas sous le point de dépôt (y = 5). **Le cas extrême (`name` 70 + `line1` 70 pour deux adresses) peut déborder comme aujourd'hui** : limite écrite, tracée à #944, sans omission de rue ni changement de taille.
5. **AC5** — **Référence au reçu** : rubrique et valeur, à leur place selon § 3.6.2, groupée comme en partie paiement, omise sans référence ; clé existante `invoice-pdf-qr-reference`, quatre locales inchangées.
6. **AC6** — Captures regardées : PDF réel des cinq cas d'AC4.
7. **AC7** — Manuel utilisateur `.tex` et PDF, CHANGELOG. Un PDF **déjà figé ne change pas** ; une facture jamais ouverte, ou un rappel régénéré, prendra le nouveau rendu : à écrire tel quel.

Hors de cette story, tracés à #944 : ciseaux et ligne de séparation, place de *Payable by*, largeur du message, très longues adresses.

## Mutations (à jouer et consigner)

M1 omettre le NPA au reçu ; M2 omettre la localité en partie paiement ; M3 ligne vide sans numéro ; M4 espace de tête sans rue ; M5 inverser NPA et localité ; M6 omettre le pays d'un débiteur étranger ; M6b omettre `LI-` ; M7 imprimer `CH-` pour un créancier suisse ; M8 tronquer au lieu de couper ; M9 omettre la référence au reçu ; M10 imprimer le débiteur à la place du créancier ; M11 `y` fixes ; M12 mot long perdu ; M13 site de dessin qui contourne la fonction.

## Gates

`test-fast.sh --ci`, `cargo build --workspace --all-targets`, **E2E complet**.

## Change Log

- 2026-11-07 — Découpée de la fiche 33-12 à la P2 de validation, F-M2, F-M3, F-M4 remédiés (critère décidable, pays, outil de preuve).
- 2026-11-07 — P3 : P3-M1 (`LI` imprimé), P3-M5 (`y` dynamiques, blocs de la partie paiement), P3-M6 (mot long) remédiés ; lecture du § 3.5.4 corrigée.
- 2026-11-07 — P4 (Opus) 1 H, 8 M, 7 L. Conduite annoncée à Guy : réduire. Table AFM, repli à 6 pt, omission de rue et pire cas sortis de la story ; forme du pays reprise des figures 19 et 20.
