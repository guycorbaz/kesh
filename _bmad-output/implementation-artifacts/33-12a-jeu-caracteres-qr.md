# Story 33-12a : le jeu de caractères de la QR-facture est celui de SIX, et l'imprimé égale le code (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, ajoutée après le kickoff : P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix »). **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Découpée à la P2 de validation (F-H1 : prémisse fausse, remédiation et correctif dans le même fichier). Rapports : `kesh-gate-logs/e3312-validate-p{1,2}.md`. Commits en `refs #941` ; à la PR : `closes #941` une fois les deux réunies.*

## Story

En tant que titulaire d'une société ou d'un contact dont le nom porte « & », je veux émettre ma QR-facture ; en tant que débiteur, je veux que le code lu par ma banque respecte la norme et que le papier dise la même chose que le code.

## Ce que dit la source (lu)

- **`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 4.1.1** : Basic Latin U+0020–U+007E, Latin-1 Supplement U+00A0–U+00FF, Latin Extended-A U+0100–U+017F, Ș U+0218, ș U+0219, Ț U+021A, ț U+021B, € U+20AC : 324 points de code. **§ 3.6.2 : les valeurs imprimées sont « exactly matching those in the Swiss QR Code ».**
- **`crates/kesh-qrbill/src/validation.rs:194-244`** se réclame d'une « Annex C », un glossaire sans jeu de caractères. Refuse à tort `& < > "`, `[ ] { } | \ ^`, l'accent grave, `~`, U+00A0, U+00AD, U+00D7, U+00F7, U+0218 à U+021B ; admet à tort U+2019 et U+2014 (Story 5-4, `8b1e479a`). Sites : `:320,342-345,356-357,461,465` ; `charset_html_unsafe_chars_rejected` (`:605`) ; fixture `:743`.
- **`pdf.rs`** : Helvetica intégrée, WinAnsi. `lopdf-0.31.0/src/encodings/mod.rs:14-19` (`filter_map`) **supprime en silence** tout caractère hors table : aujourd'hui, l'imprimé perd déjà des lettres que le code porte, ce qui viole § 3.6.2.

## Décision

**C-e3312-1 — Une seule fonction de texte émis, pour le code et pour l'imprimé de la section de paiement.** `texte_emis(&str) -> String`, dans `kesh-qrbill`, sur un texte validé : U+2019 → `'` ; U+2014 → `-` ; U+00A0 → espace ; U+00AD → retiré ; lettre hors WinAnsi → sa base par une table close et écrite (Ș → S, ț → t, ł → l, Ĳ → IJ, ŉ → n…) ; une lettre que WinAnsi porte (é, Š, Œ, €…) reste telle quelle. `build_payload` et tous les `use_text` de `draw_receipt` et `draw_payment_part` sur un texte d'adresse ou de message y passent. **L'égalité de § 3.6.2 est donc vraie par construction**, et le code reste dans le jeu. Écart assumé, à dire à Guy : un nom en Ș ou ł s'écrit S ou l dans le code comme sur le papier, jusqu'à #943 (police embarquée). Rien n'est réécrit en base. Les autres typographies (U+2018, « “ », « ” », U+2013, U+2026) restent refusées, message nommant le caractère (#942).

**Le reste du PDF n'est pas touché** : ni le corps de la facture, ni les avoirs, ni `kesh-report`, pour ne rien dégrader de ce qui s'imprime aujourd'hui. La limite est tracée à #943.

**C-e3312-2 — Ordre d'intégration** : 33-5, puis 33-12a, puis 33-12b. La signature de `check_charset` reste stable : la fonction publique de la 33-5 y délègue.

## Critères d'acceptation

1. **AC1** — Tout caractère du § 4.1.1, plus U+2019 et U+2014, passe `check_charset`. Balayage U+0000–U+024F, bornes U+00FF/U+0100, U+017F/U+0180, U+0217/U+0218, U+021B/U+021C, U+20AB/U+20AC/U+20AD, contre une **liste littérale de plages écrite dans le test depuis le PDF**.
2. **AC2** — Reste refusé tout le reste : U+0000–U+001F (dont `\n`, `\r`, `\t`), U+007F, U+0080–U+009F, U+0180–U+0217, U+021C et au-delà, U+2018, U+201C, U+201D, U+2013, U+2026.
3. **AC3** — `texte_emis` : table de paires complète (125 lettres de Latin Extended-A comprises) ; pour chaque caractère admis, sortie encodable en WinAnsi par `printpdf::lopdf` (`encode_text`), à l'intérieur du § 4.1.1, non vide sauf U+00AD, sans `?` sauf pour `?`.
4. **AC4** — **Égalité exacte, champ par champ** : `build_payload`, sur les douze champs texte (`name`, `line1`, `line2`, `postal_code`, `town` de chaque adresse, `unstructured_message`, `billing_information`), porte `texte_emis` de la valeur à la position exacte ; et les chaînes passées à `use_text` en sortent aussi : fonction pure de lignes et test d'égalité code/imprimé. Une suppression ne passe pas.
5. **AC5** — Type K : `postal_code` et `town` passent par `check_charset` et `check_len`, comme le reste.
6. **AC6** — Vacuité : les champs **obligatoires** (`name` ; `postal_code`, `town` en type S ; `line1` en type K) faits des seuls invisibles sont refusés (`kesh_core::text::is_invisible`, `validation.rs:316,328,331`). Une rue facultative qu'on laisse vide reste admise ; une rue d'invisibles sort vide.
7. **AC7** — Relecture : un code bâti avec `Max Muster & Söhne`, un `<`, un Ș, une U+00A0, se relit par `qr_decode` à la valeur de `texte_emis`. **Aucun changement du décodeur** (il lit aussi les QR de tiers, en production) ; si le test rougit, on revient au Project Lead.
8. **AC8** — Nommage : `is_six_annex_c_char` et « Annex C » cèdent la place au § 4.1.1 ; `charset_html_unsafe_chars_rejected` récrit ; fixture `:743` rétablie ; message `invoice_pdf_service.rs:498-500` à la bonne référence. Grep « Annex C », « annexe C », « SIX 2.2 » sur code, i18n, manuels, chaque site trié, **hors `crates/kesh-db/migrations/`, jamais modifiées (P8)**, hors historique.
9. **AC9** — CHANGELOG `[0.17.5]`, *Corrigé*, y compris l'écart de C-e3312-1. Un PDF **déjà figé ne change pas** ; une facture jamais ouverte, ou un rappel régénéré (`invoice_pdf_service.rs:184-188`, `issued_invoice_pdf.rs:104-111`, #502), prendra le nouveau rendu : à écrire tel quel au manuel.

## Mutations (à jouer et consigner)

M1 re-exclure `&` ; M2 retirer U+0218 ; M3 admettre U+007F ; M4 admettre U+0080 ; M5 U+2019 non normalisé dans le code ; M6 U+2014 non normalisé à l'imprimé ; M7 admettre `\n` ; M8 supprimer U+2019 au lieu de le remplacer ; M9 retirer U+00A0 ; M10 retirer U+00AD ; M11 décaler U+017F ; M12 admettre U+0180 ; M13 laisser passer une lettre hors WinAnsi ; M14 supprimer au lieu de replier ; M15 admettre U+2013 ; M16 normaliser le créancier sans le débiteur ; M17 retirer le contrôle du type K ; M18 retirer la vacuité d'un champ obligatoire.

## Gates

`test-fast.sh --ci`, `cargo build --workspace --all-targets`, **E2E complet** (un PDF visible change). Pas de `pdftotext` en test : il n'est pas en CI.

## Change Log

- 2026-11-07 — P1 (Sonnet) 0 C, 1 H, 6 M, 7 L et P2 (Opus) 0 C, 1 H, 6 M, 8 L sur la fiche unique, prémisse § 3.6.2 lue au PDF : `exactly matching`. Signal de non-convergence déclaré à Guy : F-H1 vient de ma remédiation. Conduite : simplifier (une seule fonction, reste du PDF non touché) et découper.
