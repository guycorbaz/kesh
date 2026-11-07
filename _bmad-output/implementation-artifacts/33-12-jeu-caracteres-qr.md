# Story 33-12 : le contrôle du jeu de caractères de la QR-facture est celui de SIX (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, story ajoutée après le kickoff : défaut P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix »). **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Commits en `refs #941` ; à la PR : `closes #941`.*

## Story

En tant que titulaire d'une société ou d'un contact dont le nom porte « & », je veux pouvoir émettre ma QR-facture ; et en tant que débiteur, je veux que le code que ma banque lit respecte la norme, pour que ni un refus à tort ni un code hors norme ne bloque un paiement.

## Ce que dit la source (prémisses lues)

- **`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 4.1.1** : Basic Latin U+0020–U+007E, Latin-1 Supplement U+00A0–U+00FF, Latin Extended-A U+0100–U+017F, plus Ș U+0218, ș U+0219, Ț U+021A, ț U+021B, € U+20AC. **À relire au PDF au développement**, liste recopiée au Change Log.
- **`crates/kesh-qrbill/src/validation.rs:194-244`** : `check_charset` et `is_six_annex_c_char` se réclament d'une « Annex C », qui est un glossaire et ne définit aucun jeu. Refusent à tort `& < > "`, `[ ] { } | \ ^`, l'accent grave, `~`, U+00A0, U+00AD, U+00D7, U+00F7, U+0218 à U+021B. Admettent à tort U+2019 et U+2014, décision « pragmatique » de la Story 5-4 (`8b1e479a`).
- **Sites** : `validation.rs:320,342-345,356-357,461,465` (nom, adresse, message, informations de facturation) ; test `charset_html_unsafe_chars_rejected` (`:605`), fixture `:743` ; `\n` reste exclu, il sépare les champs du code (`generator.rs`, `lines.join("\n")`).
- La 33-5 ajoute à la route d'adresse une fonction publique qui délègue à cette règle : la route suit la correction à l'intégration.

## Décision

**C-e3312-1 — U+2019 et U+2014 : normaliser à l'émission, sans refuser.** Une adresse réelle saisie avec une apostrophe typographique, comme « Rue de l’Hôpital » (cas du test `:590-594`), produit aujourd'hui une QR-facture. La refuser désormais bloquerait d'un coup une facturation qui marche, pour un gain nul : U+2019 → `'` et U+2014 → `-` dans la charge utile du code, et seulement là. La pièce imprimée est examinée : si la directive exige que le texte imprimé et le code concordent, le même texte normalisé s'imprime aussi dans la section de paiement. Aucune donnée en base n'est réécrite. Tout autre caractère hors jeu (« “ », « – », U+2603) reste refusé, avec le message actuel. *À contester en validation si la directive l'interdit.*

## Critères d'acceptation

1. **AC1** — Tout caractère du § 4.1.1 passe `check_charset`, y compris `&`, `<`, `>`, `"`, U+00A0, U+00AD, U+0218 à U+021B, € ; et sa charge utile le porte tel quel. Test exhaustif : balayage de U+0000 à U+024F plus quelques hors plage, comparé à une table écrite depuis le PDF, **non** à la fonction testée.
2. **AC2** — Reste refusé : U+0000–U+001F (dont `\n`, `\r`, `\t`), U+007F, U+0080–U+009F, U+0180–U+0217, U+021C et au-delà, sauf U+20AC, U+2019, U+2014.
3. **AC3** — U+2019 et U+2014 passent la validation et **n'atteignent jamais la charge utile** (`build_payload`). Test : charge utile entièrement balayée contre la table d'AC1, sur chaque champ texte. Aucune route d'écriture ne modifie le texte stocké.
4. **AC4** — Un PDF réel se génère et reste lisible avec `Max Muster & Söhne`, un `<`, un Ș, une U+00A0 et un U+00AD : aucun caractère sans glyphe, aucun échappement visible, code relu par le décodeur de l'import pour redonner la valeur attendue (#854). Regarde le rendu : capture du PDF au journal. Si la police n'a pas un glyphe : dis-le, ne l'élude pas.
5. **AC5** — Les nommages trompeurs tombent : `is_six_annex_c_char` et les commentaires « Annex C » cèdent la place à la référence § 4.1.1. Le test `charset_html_unsafe_chars_rejected` est récrit, sa fixture de `:743` rétablie. **Greper le symptôme** (« Annex C », « annexe C », `&` interdit) sur le dépôt : code, i18n, manuels `.tex` et PDF.
6. **AC6** — Parcours d'import ou de relecture d'une QR-facture : établir au code si une règle du même ordre y refuse ce qui devient admis, et traiter ce qui est court.
7. **AC7** — CHANGELOG `[0.17.5]`, *Corrigé* ; manuel utilisateur s'il parle des caractères ; **aucune réécriture des PDF déjà figés** (les factures validées gardent leur PDF).

## Mutations (à jouer et consigner)

M1 re-exclure `&` ; M2 retirer U+0218 ; M3 admettre U+007F ; M4 admettre U+0080 ; M5 laisser U+2019 dans la charge utile ; M6 laisser U+2014 dans la charge utile ; M7 admettre `\n` ; M8 normaliser aussi à l'écriture en base, si un chemin s'y prête.

## Limites et gates

Pas de migration. `kesh-qrbill` est aussi lu par la génération des avoirs (`pdf.rs:112-120`) : même règle, à vérifier. Gate complet, frontend non touché sauf nécessité, **E2E complet** car la génération visible change.

## Change Log

- 2026-11-07 — Fiche écrite par l'orchestrateur, à valider.
