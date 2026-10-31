# Prompt — validation P1 de la fiche v017-6 (release 0.17.0 : #832, #824), une lentille complète R+F, Opus

Tu valides la **fiche** de la Story v017-6 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11), une
**petite** story : **#832 (P4)** — le refus d'un IDE invalide répète le mot (« IDE invalide : Numéro IDE invalide : … »,
`crates/kesh-api/src/routes/onboarding.rs:538` préfixe un message de `kesh-core` qui porte déjà « Numéro IDE invalide »),
étendu par la fiche à l'IBAN et au QR-IBAN (quatre sites de plus) ; **#824 (P3, doc)** — le manuel utilisateur dit la
langue de l'interface choisie à l'onboarding, alors qu'elle est celle de l'installation (`KESH_LANG`), l'assistant ne
réglant que la langue des documents. Aucune migration.

Worktree `/home/gcorbaz/devel/kesh-0177`, branche `story/v017-6-petites-corrections`, partie de `8e126cfd`. Langue du
rapport : **français**. Modèle : **Opus**. Passe **complète**, première passe de validation, **une seule lentille** qui
tient à la fois R (la règle et sa preuve) et F (les textes et ce qu'ils affirment). **Contexte frais** : ne t'appuie sur
aucune passe antérieure, ni sur ce que la fiche dit d'elle-même.

## À lire

1. La fiche `_bmad-output/implementation-artifacts/v017-6-petites-corrections.md`, **en entier**.
2. Le registre `_bmad-output/implementation-artifacts/v017-6-choix-autonomes.md` (C-v017-6-1 à 8).
3. Les issues, **en lecture seule** : `gh issue view 832 --json title,body,comments`, idem `824` et `710`.
4. Le code : `crates/kesh-core/src/errors.rs`, `crates/kesh-core/src/types/{che_number,iban,qr_iban}.rs`,
   `crates/kesh-api/src/routes/{onboarding,bank_accounts,contacts,supplier_invoices}.rs`,
   `crates/kesh-api/src/errors.rs` (bras `Validation`), `crates/kesh-api/src/routes/i18n.rs`,
   `crates/kesh-api/src/config.rs` (`KESH_LANG`), `crates/kesh-api/src/routes/{invoice_email,issued_invoice_pdf}.rs` ;
   les tests `crates/kesh-api/tests/{onboarding_path_b_e2e,bank_accounts_e2e}.rs` ;
   `frontend/src/routes/onboarding/+page.svelte` (`chooseLanguage`, `loadI18nMessages`).
5. Le **manuel** : `docs/manual/fr/user-manual.tex` (§ *Choix de la langue*, l. 143-168 ; l. 183-206 ; l. 3732),
   `docs/manual/fr/admin-manual.tex` (l. 757, l. 1069, l. 1418), `README.md:53`. **Le PDF aussi**, lu aplati et vers la
   sortie standard seulement : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
6. Pour contexte : `CLAUDE.md` du worktree (*Review Iteration Rule*, *Issue Tracking Rule*, *Propagation post-patch*,
   *Recompter ses propres comptes rendus*) ; `docs/agents-orchestration.md` ;
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.

## Axes à exercer

**La règle et sa preuve**

- **Prémisses** : chaque `fichier:ligne` de la fiche (§ *Ce que le code dit*, 1 à 10) dit-il ce que la fiche lui fait
  dire ? `CheNumber::new`, `Iban::new`, `QrIban::new` ne rendent-ils que des `CoreError` de la forme supposée ? Le
  message exact prescrit par AC-2 (« Numéro IDE invalide : doit commencer par CHE » pour `"INVALID"` ; « IBAN invalide :
  trop court » pour `"BAD"`) est-il bien celui que produit le code (normalisation, majuscules, ordre des contrôles) ?
- **Inventaire clos des sites (§ 3)** : refais le `grep` toi-même, plus large (`format!` sur `Validation`, `BadRequest`,
  `{e}`, `{err}`, `.to_string()`, y compris `kesh-api/src/services`, `exports`, `mcp`, et les crates voisines). Existe-t-il un
  site qui préfixe le `Display` d'un `CoreError` ou d'une erreur qui répète déjà le mot, et que la table omet ? Chaque
  ligne « hors classe » l'est-elle vraiment (lis l'erreur interpolée) ?
- **AC-2 et AC-3** : chaque assertion rougit-elle sous la mutation annoncée, et seulement elle ? Le comptage de
  « IBAN invalide » (AC-2 d) est-il vrai pour les deux messages neufs et faux pour les anciens ? Les entrées
  `"BAD"`/`"INVALID"` atteignent-elles le contrôle visé (ordre des refus de `create_bank_account`, gardes préalables de
  `set_coordinates`, étape de l'assistant exigée avant la banque) ? Une régression probable reste-t-elle sans mutation ?
- **Les tests existants** (`starts_with`, code seul) restent-ils verts ; la fiche dit-elle vrai qu'ils sont muets ?
- **Le périmètre** : un défaut voisin non mécanique est-il intégré à tort ? Un défaut mécanique est-il renvoyé à tort ?
  Les deux commentaires d'issue prévus sont-ils justifiés ? Un point écarté sans issue (« on trace toujours») ?
- **Les règles du dépôt** : aucune migration, aucune écriture sur les données, aucun mot-clé de fermeture dans les
  commits prescrits, gate complet au dernier commit de code, aucune donnée réelle.

**Les textes et ce qu'ils affirment**

- **Le manuel (AC-5 à AC-8)** — axe obligatoire : la phrase prescrite est-elle **vraie au code** (interface dans la
  locale de `KESH_LANG` ; étape 0 = `instance_language` = langue des documents ; défaut `fr`) ? L'inventaire § 9 est-il
  clos : cherche d'autres phrases du `.tex` **et du PDF aplati** qui disent ou impliquent que la langue est choisie à
  l'assistant (y compris dans le manuel d'administration, la brochure, le site, les README), ou, à l'inverse, qui
  deviennent fausses ou muettes après la récriture ? La `keshwarning` conservée (#710, « avant la 0.11.0 ») et la phrase
  de l'étape 4 (l. 202) restent-elles vraies et compatibles avec la phrase neuve ? Le `grep` d'AC-8 est-il bien formé
  (greper la valeur, non la formulation) ? Le renvoi au manuel d'administration est-il exact ?
- **Le choix de ne pas toucher `admin-manual.tex:757` (C-v017-6-5)** tient-il, ou le manuel d'administration est-il
  réellement muet sur l'essentiel ?
- **CHANGELOG (AC-9)** : les deux puces sont-elles vraies, bien rangées (*Corrigé*), sans fermeture de ticket, sans
  toucher l'intro ? Le registre et la fiche disent-ils pareil ?
- **Les décomptes** (cinq sites, quatre de plus, quatre phrases fausses, numéros de ligne) : recompte-les depuis la source
  (`grep -c`, `sed -n`).

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à
`/home/gcorbaz/devel/kesh-gate-logs/v0176-validate-p1.md`. Il contient :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne` de la fiche, du registre ou du code), **la ligne lue citée** (sortie `grep -nF` ou extrait copié)
   qui fonde l'affirmation, et le remède proposé. Une affirmation de présence ou d'absence sans sortie `grep -nF` copiée
   ne compte pas.
2. **Les axes exercés** et **les axes non exercés**, un par un, avec, pour chaque axe exercé, ce que tu as lu ou
   cherché. Un « 0 finding » sans cette liste ne clôt rien et ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`, sous
  aucune forme.
- Aucune commande `gh` qui écrit (`issue create|edit|close|comment`, `pr …`) ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `npx`, ni `make`, ni
  `latexmk`, ni `python3`, ni `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
