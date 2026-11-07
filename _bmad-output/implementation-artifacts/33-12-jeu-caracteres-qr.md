# Story 33-12 : le contrôle du jeu de caractères de la QR-facture est celui de SIX (#941)

Status: ready-for-dev

*Fiche rédigée le 2026-11-07 par l'orchestrateur, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`. Release **0.17.5**, Epic 33, story ajoutée après le kickoff : défaut P1 de conformité trouvé à la revue de la 33-5, dans le cadre donné par Guy (« quelques bugs de ton choix ») ; porte aussi le NPA et la localité absents du reçu. **Aucune migration.** Registre : `e33-choix-autonomes-e3312.md`. Commits en `refs #941` ; à la PR : `closes #941`.*

## Story

En tant que titulaire d'une société ou d'un contact dont le nom porte « & », je veux pouvoir émettre ma QR-facture ; et en tant que débiteur, je veux que le code que ma banque lit respecte la norme, pour que ni un refus à tort ni un code hors norme ne bloque un paiement.

## Ce que dit la source (prémisses lues)

- **`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 4.1.1** : Basic Latin U+0020–U+007E, Latin-1 Supplement U+00A0–U+00FF, Latin Extended-A U+0100–U+017F, plus Ș U+0218, ș U+0219, Ț U+021A, ț U+021B, € U+20AC. **À relire au PDF au développement**, liste recopiée au Change Log.
- **`crates/kesh-qrbill/src/validation.rs:194-244`** : `check_charset` et `is_six_annex_c_char` se réclament d'une « Annex C », qui est un glossaire et ne définit aucun jeu. Refusent à tort `& < > "`, `[ ] { } | \ ^`, l'accent grave, `~`, U+00A0, U+00AD, U+00D7, U+00F7, U+0218 à U+021B. Admettent à tort U+2019 et U+2014, décision « pragmatique » de la Story 5-4 (`8b1e479a`).
- **Sites** : `validation.rs:320,342-345,356-357,461,465` (nom, adresse, message, informations de facturation) ; test `charset_html_unsafe_chars_rejected` (`:605`), fixture `:743` ; `\n` reste exclu, il sépare les champs du code (`generator.rs`, `lines.join("\n")`).
- La 33-5 ajoute à la route d'adresse une fonction publique qui délègue à cette règle : la route suit la correction à l'intégration.

## Décisions

**C-e3312-1 — U+2019 et U+2014 : normaliser dans le code QR, sans refuser.** Une adresse réelle saisie avec une apostrophe typographique, comme « Rue de l’Hôpital » (cas du test `:590-594`), produit aujourd'hui une QR-facture ; la refuser bloquerait d'un coup une facturation qui marche. U+2019 → `'` et U+2014 → `-`, **dans `build_payload` seul**, sur chaque champ texte émis. La directive n'exige pas que l'imprimé et le code concordent (validation P1, axe 2) : l'impression garde le texte d'origine, que WinAnsi sait imprimer. Rien n'est réécrit en base. Les autres typographies (U+2018, « “ », « ” », U+2013, U+2026) restent refusées, avec le message qui nomme le caractère ; l'orchestrateur en ouvre l'issue.

**C-e3312-2 — À l'impression, un caractère sans glyphe ne disparaît plus : il se replie.** `pdf.rs` emploie Helvetica intégrée, encodée en WinAnsi, et `lopdf-0.31.0/src/encodings/mod.rs:14-19` (`string_to_bytes`, `filter_map`) **supprime en silence** tout caractère hors table. Admettre tout le § 4.1.1 sans rien changer ferait coller des mots et perdre des lettres, ce qui vaut déjà aujourd'hui pour une partie de Latin Extended-A. Un seul helper, par lequel passe **tout texte d'utilisateur du module** (`use_text` de `pdf.rs`, facture comprise) : U+00A0 → espace ; U+00AD → retiré ; lettre hors WinAnsi → sa lettre de base par une table close et écrite (Ș → S, ț → t, ł → l…) ; tout ce qui resterait → `?`, jamais rien. Une lettre que WinAnsi porte (é, Š, Œ…) reste telle quelle. **Le code QR garde le texte exact**. Embarquer Liberation Sans, admise au § 3.4 et plus juste, est un chantier distinct : l'orchestrateur en ouvre l'issue.

**C-e3312-3 — Ordre d'intégration** : la 33-5 d'abord, la 33-12 ensuite. La signature de `check_charset` reste stable : la fonction publique de la 33-5 y délègue. Seuls le nom interne et la table changent.

## Critères d'acceptation

1. **AC1** — Tout caractère du § 4.1.1 (324 points de code) et U+2019, U+2014 passent `check_charset`. Test exhaustif, U+0000 à U+024F, bornes U+00FF/U+0100, U+017F/U+0180, U+0217/U+0218, U+021B/U+021C, U+20AB/U+20AC/U+20AD, comparé à une **liste littérale de plages écrite dans le test depuis le PDF**, plus les deux exceptions de C-e3312-1, jamais dérivée de la fonction.
2. **AC2** — Reste refusé tout le reste : U+0000–U+001F (dont `\n`, `\r`, `\t`), U+007F, U+0080–U+009F, U+0180–U+0217, U+021C et au-delà, sauf les exceptions.
3. **AC3** — **Égalité exacte**, champ par champ : une charge utile bâtie avec U+2019 et U+2014 sur chacun des douze champs texte émis (inventaire au rapport `e3312-validate-p1.md`, M2) porte `'` et `-` à la position exacte ; un balayage de « ne contient pas » ne suffit pas, une suppression le passerait. Un caractère du § 4.1.1 y arrive inchangé. L'impression, elle, garde U+2019.
4. **AC4** — Le type K : `postal_code` et `town`, aujourd'hui émis sans `check_charset` ni `check_len`, sont contrôlés comme les autres, ou le code établit qu'ils sont toujours vides, ligne lue citée. Aucun champ texte émis n'échappe au contrôle (inventaire).
5. **AC5 (C-e3312-2)** — Pour chaque caractère du § 4.1.1, le helper rend une chaîne encodable en WinAnsi, testée contre la table réelle de `lopdf`, et non vide sauf U+00AD ; `Ș` rend `S`, `é` rend `é`. Un PDF réel avec `Max Muster & Söhne`, un `<`, un Ș, une U+00A0, un U+00AD s'extrait par `pdftotext` avec le texte attendu ; capture regardée au journal. Le code se relit par `qr_decode` et redonne la valeur exacte ; si le décodage n'est pas fixé en UTF-8 (`qr_decode.rs:253-260`, `CHARACTER_SET`), corrige-le ou prouve qu'il tient sur ces caractères.
6. **AC6 — NPA et localité imprimés.** Défaut trouvé à la validation : `pdf.rs:765-769, 806-810, 867-871, 935-939` n'imprime que `name`, `line1`, `line2`, or la génération est toujours de type S (`invoice_pdf_service.rs:294,311`), où `line1` est la rue et `line2` le numéro. **Le reçu et la partie paiement omettent donc NPA et localité, et séparent la rue du numéro.** Reçu et partie paiement, créancier et débiteur : en type S, `rue numéro` puis `NPA localité` ; un champ absent ne laisse ni ligne vide ni espace en tête. Tests sur le texte extrait d'un PDF réel, y compris sans numéro ; capture. Vérifie les positions : rien ne déborde de la section de paiement ni n'y chevauche le bloc suivant.
7. **AC7** — Vacuité : un nom, une rue ou une localité faits du seul U+00AD (ou d'autres invisibles admis) sont refusés, `kesh_core::text::is_invisible` à l'appui (`validation.rs:316,328,331`).
8. **AC8** — Les nommages trompeurs tombent : `is_six_annex_c_char` et les commentaires « Annex C » cèdent la place au § 4.1.1 ; `charset_html_unsafe_chars_rejected` est récrit ; la fixture de `:743` rétablie ; le message « non autorisé par SIX 2.2 » (`invoice_pdf_service.rs:498-500`) cite la bonne version. **Greper le symptôme** (« Annex C », « annexe C », « SIX 2.2 », `&`) : code, i18n, manuels `.tex` et PDF.
9. **AC9** — CHANGELOG `[0.17.5]`, *Corrigé* ; manuel utilisateur s'il parle des caractères ou de la section de paiement ; **aucune réécriture des PDF déjà figés** : les factures validées gardent leur PDF, ce que le manuel et les notes de release doivent dire (une facture déjà émise garde son reçu sans NPA ni localité, et le QR, lui, les porte).

Constats de validation à ne pas traiter ici : NFD (L3), `EPD\n` et autres écarts de charge utile (L4) : à relire au développement, issue demandée si cela déborde.

## Mutations (à jouer et consigner)

M1 re-exclure `&` ; M2 retirer U+0218 ; M3 admettre U+007F ; M4 admettre U+0080 ; M5 laisser U+2019 dans la charge utile ; M6 laisser U+2014 dans la charge utile ; M7 admettre `\n` ; M8 supprimer U+2019 au lieu de le remplacer ; M9 retirer U+00A0 ; M10 retirer U+00AD ; M11 décaler la borne U+017F ; M12 admettre U+0180 ; M13 helper qui laisse passer un caractère hors WinAnsi ; M14 helper qui supprime au lieu de replier ; M15 omettre le NPA au reçu ; M16 omettre la localité à la partie paiement ; M17 ligne vide sans numéro.

## Limites et gates

Pas de migration. Les avoirs n'ont ni QR ni contrôle du jeu : non concernés, hors passage de leur texte par le helper. Rien à l'import : le parseur ne contrôle que des longueurs. Base du worktree : `5d4add7c` ou sa suite. Gate complet, frontend non touché sauf nécessité, **E2E complet** car un PDF visible change.

## Change Log

- 2026-11-07 — Fiche écrite par l'orchestrateur, à valider.
- 2026-11-07 — Validation P1 (Sonnet, `e3312-validate-p1.md`, 0 C, 1 H, 6 M, 7 L) remédiée par l'orchestrateur, prémisses de H1 et M5 lues au code : C-e3312-1 tranchée, C-e3312-2 et 3 ajoutées, AC refondus, M8 remplacée.
