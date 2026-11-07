# Story 33-12b : le reçu et la partie paiement impriment l'adresse entière (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, ajoutée après le kickoff : P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix »). **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Découpée à la P2 de validation (F-H1 : prémisse fausse, remédiation et correctif dans le même fichier). Rapports : `kesh-gate-logs/e3312-validate-p{1,2}.md`. Commits en `refs #941` ; à la PR : `closes #941` une fois les deux réunies.*

## Story

En tant que débiteur qui lit ou archive une QR-facture, je veux trouver sur le papier le NPA et la localité du créancier et les miens, et la référence au reçu.

## Ce que le code dit (lu)

- La génération est toujours de type S (`invoice_pdf_service.rs:294,311`), où `line1` est la rue et `line2` le numéro. `pdf.rs:765-769, 806-810, 867-871, 935-939` n'imprime que `name`, `line1`, `line2` : **ni NPA ni localité, et la rue séparée du numéro**, au créancier et au débiteur, sur le reçu et la partie paiement. Le code QR les porte. Aucun test n'en dit rien : les jeux d'essai PDF sont de type K (`pdf.rs:1293,1302`, `golden_test.rs:27,36`), et le dépôt ne mesure que des tailles (`pdf.rs:1386-1388`).
- `docs/six-references/ig-qr-bill-v2.4-en.pdf` : **§ 3.6.2** (reçu), « Because of the limited space, it is permitted to […] omit the street name and building number » ; code pays pour un créancier hors de Suisse et du Liechtenstein, et pour un débiteur hors de Suisse ; le nom peut tenir sur deux lignes. § 3.5.4 de même pour la partie paiement ; minimum 6 pt en partie paiement. Ce paragraphe exige une rubrique **Référence** au reçu : `draw_receipt` n'en a aucune (seul `:876`, en partie paiement).
- Dépend de **33-12a** : les lignes sortent de `texte_emis`.

## Critères d'acceptation

1. **AC1** — Une fonction pure `lignes_adresse(&Address, …) -> Vec<String>`, seule source des deux colonnes : type S, `rue numéro` puis `NPA localité` ; un champ absent ne laisse ni ligne vide ni espace de tête ; type K inchangé. Tests sur cette fonction, plus au moins un sur le contenu d'un PDF réel par `printpdf::lopdf` (`extract_text`), non par `pdftotext`, absent de la CI.
2. **AC2** — Pays : créancier hors CH et LI, débiteur hors CH : `XX-NPA localité`. **Forme à établir aux échantillons de `docs/six-references/` et à la Style Guide citée** ; si elle n'y est pas, cette forme, notée comme choix au registre.
3. **AC3** — Largeur, critère décidable : une fonction de largeur d'après les métriques AFM d'Helvetica ; aucune ligne, à la taille employée, ne dépasse sa colonne (reçu : 52 mm ; partie paiement : largeur à relever au code, à droite du QR). Passé la largeur : coupe à une frontière de mot, jamais de troncature silencieuse. Sur le reçu, si la hauteur manque, omets rue et numéro, ce que § 3.6.2 permet. Cas d'essai aux bornes : `name` 70, `line1` 70, `line2` 16, `postal_code` 16, `town` 35.
4. **AC4** — Hauteur : le plus long cas, deux adresses comprises, ne chevauche pas le bloc suivant (montant, point de dépôt) : calcul des `y` écrit au test.
5. **AC5** — **Référence au reçu** : rubrique et valeur à leur place, groupée comme en partie paiement, omise sans référence ; quatre locales inchangées si la clé `invoice-pdf-qr-reference` existe.
6. **AC6** — Captures regardées, thème clair sans objet : PDF réel, adresse suisse courte, adresse à la borne, débiteur étranger, sans référence.
7. **AC7** — Manuel utilisateur `.tex` et PDF, CHANGELOG. Un PDF **déjà figé ne change pas** ; une facture jamais ouverte, ou un rappel régénéré, prendra le nouveau rendu : à écrire tel quel.

Hors de cette story : ciseaux et ligne de séparation (§ 3.7), `kesh-report` : l'orchestrateur en ouvre les issues.

## Mutations (à jouer et consigner)

M1 omettre le NPA au reçu ; M2 omettre la localité en partie paiement ; M3 ligne vide sans numéro ; M4 espace de tête sans rue ; M5 inverser NPA et localité ; M6 omettre le pays d'un débiteur étranger ; M7 imprimer `CH-` pour un créancier suisse ; M8 tronquer au lieu de couper ; M9 omettre la référence au reçu ; M10 imprimer le débiteur à la place du créancier.

## Gates

`test-fast.sh --ci`, `cargo build --workspace --all-targets`, **E2E complet**.

## Change Log

- 2026-11-07 — Découpée de la fiche 33-12 à la P2 de validation, F-M2, F-M3, F-M4 remédiés (critère décidable, pays, outil de preuve).
