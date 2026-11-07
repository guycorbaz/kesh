# Story 33-12a : le jeu de caractères de la QR-facture est celui de SIX, et l'imprimé égale le code (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, ajoutée après le kickoff : P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix »). **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Découpée à la P2 de validation (F-H1 : prémisse fausse, remédiation et correctif dans le même fichier). Rapports : `kesh-gate-logs/e3312-validate-p{1,2,3,4}.md`. Le registre se crée au développement. Commits en `refs #941` ; à la PR : `closes #941` une fois les deux réunies.*

## Story

En tant que titulaire d'une société ou d'un contact dont le nom porte « & », je veux émettre ma QR-facture ; en tant que débiteur, je veux que le code lu par ma banque respecte la norme et que le papier dise la même chose que le code.

## Ce que dit la source (lu)

- **`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 4.1.1** : Basic Latin U+0020–U+007E, Latin-1 Supplement U+00A0–U+00FF, Latin Extended-A U+0100–U+017F, Ș U+0218, ș U+0219, Ț U+021A, ț U+021B, € U+20AC : 324 points de code. **§ 3.6.2 : les valeurs imprimées sont « exactly matching those in the Swiss QR Code ».**
- **`crates/kesh-qrbill/src/validation.rs:194-244`** se réclame d'une « Annex C », un glossaire sans jeu de caractères. Refuse à tort `& < > "`, `[ ] { } | \ ^`, l'accent grave, `~`, U+00A0, U+00AD, U+00D7, U+00F7, U+0218 à U+021B ; admet à tort U+2019 et U+2014 (Story 5-4, `8b1e479a`). Sites : `:320,342-345,356-357,461,465` ; `charset_html_unsafe_chars_rejected` (`:605`) ; fixture `:743`.
- **`pdf.rs`** : Helvetica intégrée, WinAnsi. `lopdf-0.31.0/src/encodings/mod.rs:14-19` (`filter_map`) **supprime en silence** tout caractère hors table : aujourd'hui, l'imprimé perd déjà des lettres que le code porte, ce qui viole § 3.6.2.

## Décisions

**C-e3312-1 — Deux niveaux, un seul sens : le code d'abord, l'imprimé en dérive.**
- `texte_code(&str)`, sur un texte validé : U+2019 → `'` ; U+2014 → `-` ; U+00A0 → espace ; U+00AD → retiré ; puis rognage des blancs de bordure, si bien qu'une rue d'invisibles sort vide et qu'un NPA ne commence pas par un blanc. Aucune lettre touchée, aucune sortie plus longue que l'entrée. `build_payload` y passe pour chaque champ texte.
- `texte_imprime(&str) = repli(texte_code(s))` : lettre hors WinAnsi (121 de Latin Extended-A, plus Ș ș Ț ț) → sa base par une table close et écrite (Č → C, ł → l, Ș → S, Ĳ → IJ…) ; tout le reste inchangé. Les lettres que WinAnsi porte (é, Š, Œ, €…) restent telles quelles.
- **Le code garde donc Čolić comme aujourd'hui.** Dans la section de paiement, l'imprimé égale le code sur les champs qu'il imprime, sauf sur ces 125 lettres, qui jusqu'ici disparaissaient du papier (Čolić s'imprime « oli ») et s'impriment désormais par leur base (« Colic ») : écart résiduel à § 3.6.2, assumé, dit à Guy et au manuel, levé par #943 (police embarquée). NPA et localité n'arrivent à l'imprimé qu'avec la 33-12b.
- Rien n'est réécrit en base. U+2018, « “ », « ” », U+2013, U+2026 restent refusés (#942).

**Le reste du PDF n'est pas dans cette story** (corps de la facture, avoirs, `kesh-report`) : hors du périmètre, leurs trous actuels restent et sont tracés à #943.

**C-e3312-2 — Ordre d'intégration** : 33-5, puis 33-12a, puis 33-12b. La signature de `check_charset` reste stable : la fonction publique de la 33-5 y délègue.

## Critères d'acceptation

1. **AC1** — Tout caractère du § 4.1.1, plus U+2019 et U+2014, passe `check_charset`. Balayage U+0000–U+024F, bornes U+00FF/U+0100, U+017F/U+0180, U+0217/U+0218, U+021B/U+021C, U+20AB/U+20AC/U+20AD, contre une **liste littérale de plages écrite dans le test depuis le PDF**.
2. **AC2** — Reste refusé tout le reste : U+0000–U+001F (dont `\n`, `\r`, `\t`), U+007F, U+0080–U+009F, U+0180–U+0217, U+021C et au-delà, U+2018, U+201C, U+201D, U+2013, U+2026.
3. **AC3** — `texte_code` : paires littérales pour U+2019, U+2014, U+00A0, U+00AD ; tout autre caractère admis inchangé ; longueur de sortie ≤ longueur d'entrée ; sortie dans le § 4.1.1.
4. **AC4** — `texte_imprime` : table de paires complète, 121 + 4 ; pour chaque caractère admis, sortie encodable en WinAnsi (`printpdf::lopdf`, `encode_text`, la table étant privée), non vide sauf U+00AD, sans `?` sauf pour `?`.
5. **AC5** — **Égalité exacte, champ par champ** : `build_payload`, sur les douze champs texte (`name`, `line1`, `line2`, `postal_code`, `town` de chaque adresse, `unstructured_message`, `billing_information`), porte `texte_code` de la valeur à la position exacte.
6. **AC6** — **Tous les sites de dessin** d'un texte d'adresse dans `draw_receipt` et `draw_payment_part`, soit douze avec les quatre `name`, plus le message de la partie paiement, passent par une fonction pure unique, dont la sortie est `texte_imprime`. La preuve principale porte sur cette fonction. **Preuve sur un PDF réel à établir à l'étape 0** : `extract_text` de `lopdf` lit en StandardEncoding et rend `Shne` pour `Söhne` (P4-H1, lu à la source, non exécuté) ; passe par `Content::decode` puis `decode_text(Some("WinAnsiEncoding"), …)` avec une **sonde positive**. Si cet outil ne marche pas, dis-le : la mutation M6 se joue alors par relecture, déclarée telle. `billing_information` ne s'imprime pas.
7. **AC7** — Type K : `postal_code` et `town` passent par `check_charset` et `check_len`, comme le reste.
8. **AC8** — Vacuité : les champs **obligatoires** (`name` ; `postal_code`, `town` en type S ; `line2` en type K) faits des seuls invisibles sont refusés (`kesh_core::text::is_invisible`, `validation.rs:316,328,331,351`). Une rue facultative vide reste admise ; une rue d'invisibles sort vide.
9. **AC9** — Relecture, test dans `kesh-api` où vit `qr_decode` : un code bâti avec `Max Muster & Söhne`, un `<`, un Ș, une U+00A0, se relit à la valeur de `texte_code`. **Aucun changement du décodeur** (il lit aussi les QR de tiers, en production) ; si le test rougit, on revient au Project Lead.
10. **AC10** — Nommage : `is_six_annex_c_char` et « Annex C » cèdent la place au § 4.1.1 ; `charset_html_unsafe_chars_rejected` récrit ; fixture `:743` rétablie ; message `invoice_pdf_service.rs:498-500` à la bonne référence. Grep « Annex C », « annexe C », « SIX 2.2 » sur code, i18n, manuels, chaque site trié, **hors `crates/kesh-db/migrations/`, jamais modifiées (P8)**, hors historique.
11. **AC11** — CHANGELOG `[0.17.5]`, *Corrigé*, et manuel utilisateur `.tex` + PDF, avec l'écart de C-e3312-1. Un PDF **déjà figé ne change pas** ; une facture jamais ouverte, ou un rappel régénéré (`invoice_pdf_service.rs:184-188`, `issued_invoice_pdf.rs:104-111`, #502), prendra le nouveau rendu : à écrire tel quel.

## Mutations (à jouer et consigner)

M1 re-exclure `&` ; M2 retirer U+0218 ; M3 admettre U+007F ; M4 admettre U+0080 ; M5 U+2019 non normalisé dans le code ; M6 un site de dessin qui contourne `texte_imprime` ; M7 admettre `\n` ; M8 supprimer U+2019 au lieu de le remplacer ; M9 retirer U+00A0 ; M10 retirer U+00AD ; M11 décaler U+017F ; M12 admettre U+0180 ; M13 laisser passer une lettre hors WinAnsi à l'imprimé ; M14 supprimer au lieu de replier ; M14b replier aussi dans le code ; M15 admettre U+2013 ; M16 normaliser le créancier sans le débiteur ; M17 retirer le contrôle du type K ; M18 retirer la vacuité d'un champ obligatoire.

## Gates

`test-fast.sh --ci`, `cargo build --workspace --all-targets`, **E2E complet** (un PDF visible change). Pas de `pdftotext` en test : il n'est pas en CI.

## Change Log

- 2026-11-07 — P1 (Sonnet) 0 C, 1 H, 6 M, 7 L et P2 (Opus) 0 C, 1 H, 6 M, 8 L sur la fiche unique, prémisse § 3.6.2 lue au PDF : `exactly matching`. Signal de non-convergence déclaré à Guy : F-H1 vient de ma remédiation. Conduite : simplifier (reste du PDF non touché) et découper.
- 2026-11-07 — P3 (Sonnet) 0 C, 0 H, 6 M, 12 L sur les deux fiches. P3-M2 (replier le code dégrade Čolić) et P3-M3 (`Ĳ → IJ` allonge après validation) : C-e3312-1 refondue, le code ne touche plus aux lettres. P3-M4 : preuve sur PDF réel ici (AC6). Les LOW courts repris.
- 2026-11-07 — P4 (Opus) 1 H, 8 M, 7 L, dont sept au-dessus de LOW nés de la remédiation P3. Conduite annoncée à Guy : ne plus rapiécer, réduire. AC6 refondu sur une fonction pure, `texte_code` rogne. Signal de non-convergence déclaré.
