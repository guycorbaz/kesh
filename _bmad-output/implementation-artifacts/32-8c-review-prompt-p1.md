# Prompt — revue de code P1 de la Story 32-8c (l'écran des envois du serveur MCP), trois lentilles, Opus

Tu relis le développement de la **sous-partie c de la Story 32-8** de Kesh (comptabilité suisse ; frontend SvelteKit,
backend Rust/Axum, MariaDB) : *l'écran `/admin/mcp/sends` où un Administrateur voit ce qu'un agent a demandé au serveur
MCP sous une clé dont la validation est requise, et l'envoie tel quel, le modifie avant de l'envoyer, ou le refuse ;
l'E2E de bout en bout ; la recette visuelle ; le manuel d'administration (section `sec:mcp-envois`) et les textes que la
table `mcp_sends` rend faux ; le CHANGELOG.* La propriété centrale de la Story 32-8 est « **ce que l'utilisateur voit
est exactement ce qui part**, octet pour octet, et rien ne part sans lui » (fiche, § *La propriété centrale*) : la
32-8c en tient le dernier maillon — la valeur de la `textarea` est la chaîne reçue, et « modifié » envoie cette valeur
sans transformation. Un défaut ici est un texte qui part différent de ce qui a été vu, ou un texte qui part sans avoir
été vu.

Worktree `/home/gcorbaz/devel/kesh-32-8a`, branche `story/32-8c`. Le diff à relire, **aplati** :

```sh
git -C /home/gcorbaz/devel/kesh-32-8a diff origin/main...HEAD --stat
git -C /home/gcorbaz/devel/kesh-32-8a diff origin/main...HEAD -- frontend crates docs CHANGELOG.md
```

Langue du rapport : français. Modèle : **Opus**. **Contexte frais** : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la 32-8c est en revue.** La table et la sauvegarde (32-8a), le prédicat des caractères cachés (32-8b0), la
poignée serveur et les routes `/api/v1/admin/mcp/sends…` (32-8b) sont fusionnés et relus ; l'écran *Serveur MCP*
(`/admin/mcp`) et la section *Le serveur MCP* du manuel sont à la 32-7, **non fusionnée** ; les outils de données à la
32-9 ; les prompts à la 32-10a. Leur absence n'est pas un finding. Un défaut **du serveur** se signale (sévérité
comprise), mais sa correction relève d'une autre story. Les LOW déjà tracés de la 32-8b sont à l'issue **#768** ; le
repli i18n sans garde générale à **#758** — tu ne lances pas `gh`, la fiche et le registre les citent.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` — § *La propriété centrale*, la **Partie C**
(AC-C1 à AC-C11), l'*inventaire clos des chemins* (D1 à D27 : la 32-8c en écrit plusieurs au manuel — D3, D4, D4 bis,
D17, D18, D22), les tâches *32-8c* (T-C0 à T-C3), les mutations **MC1 à MC6**, le § *Gates* (32-8c), *Ce que les gardes
ne voient pas*, le § *Le manuel (axe de revue)*, le § *À recontrôler à l'étape 0* (note sur **24 576** octets,
constat **P3-R6**) — puis le **Dev Agent Record** et les entrées *32-8c* du **Change Log** ; le registre
`epic-32-choix-autonomes-328.md`, **C-32-8-70 à C-32-8-77** (et C-32-8-20, C-32-8-58 cités par eux). Le Dev Agent
Record déclare des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les
journaux `/home/gcorbaz/devel/kesh-gate-logs/328c-*` (lecture seule) : `328c-gate-complet.log`,
`328c-gate-frontend-final.log`, `328c-e2e.log`, `328c-mutations.log`, `328c-mutations-resultats.json`,
`328c-make-fr.log`.

**Les captures** de la recette visuelle (`CLAUDE.md` § *Recette visuelle*, thème clair) sont jointes :
`/home/gcorbaz/devel/kesh-gate-logs/328c-captures/` — `01-liste-vide.png`, `02-liste-chaque-etat.png`,
`03-detail-en-attente.png`, `04-detail-en-modification.png`, `05-dialogue-de-refus.png`, `06-contenu-piege.png`,
`07-modification-refusee-caractere-invisible.png`, `08-conflit-deja-decide.png`, `09-valide-modifie.png`,
`10-entree-de-navigation.png`. **Ouvre-les** (outil de lecture d'images) : un test vérifie qu'un élément existe, non
qu'on le voit. Confronte chacune à l'AC-C9 et à ce que le manuel dit de l'écran.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`),
  aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx` (ni `vitest`, ni `playwright`, ni `svelte-check`), ni `bash -c`/`sh -c` d'un script, ni `jq` ;
  ni **`scripts/prepare-release.sh`** (il bumpe les versions du workspace : il est **interdit**), ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `xelatex`, ni `cargo` (aucune
  sous-commande, `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et l'outil de lecture d'images pour les captures.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) **copiée** dans
  le rapport. Un « 0 finding » sans la liste des axes exercés **et non exercés** ne compte pas comme passe.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Les trois axes obligatoires, pour chaque lentille

1. **Ce qui part de l'écran.** Pour chacun des trois gestes, quel corps exact part vers
   `POST /api/v1/admin/mcp/sends/{id}/decision` ? Le serveur lit un `content` **absent ou nul** comme « tel quel »
   (`#[serde(default)]`, `routes/mcp_envois.rs:122-127` ; #768 point 4) : l'écran peut-il, par un chemin quelconque,
   envoyer `"content": null`, `undefined` sérialisé, ou une valeur **transformée** (trim, normalisation, fin de ligne,
   `bind:value` qui réécrit) ? Le bouton « tel quel » / « version modifiée » suit-il **la valeur** et non un drapeau
   d'édition ? Une version ramenée à la valeur reçue part-elle bien « tel quel » ?
2. **Ce que l'écran montre.** La valeur de la `textarea` est-elle **la chaîne reçue, sans transformation** (une valeur
   qui commence par `\n`, des tabulations, des blancs finaux, une suite `\u{202E}` littérale) ? Le contenu et le résumé
   ne passent-ils que par la **valeur** d'un champ ou l'**interpolation** de Svelte — jamais une insertion de HTML
   brut, ni un attribut `title`/`aria-*` qui recevrait le contenu sans échappement ? L'avis permanent dit-il **le texte
   de l'AC-C3** ? Ce que montrent les captures est-il ce que dit le manuel ?
3. **Le manuel, contre le code et contre le PDF** (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*).
   Chaque affirmation de la section `\subsection{Les envois du serveur MCP}` (`docs/manual/fr/admin-manual.tex`) et des
   phrases corrigées (`:1414` remise à zéro, `:2230` le fichier est un secret, `:2237` ce que porte le fichier, `:2249`
   les tables de la 0.16.0 ; `user-manual.tex` ≈3027-3035 l'export global, ≈3153-3158 l'auteur d'un appel MCP au
   journal) est-elle **vraie au code** — l'échéance de 24 h et ce qu'elle interdit, la purge paresseuse, la limite de
   20 (souple), les 24 576 octets **après** le rendu visible et le 1 Kio du résumé, les codes `\u{…}`, les refus 403/401
   par rôle et par clé, les issues qui partent sans validation (D3, D4, D4 bis), la clé dispensée, la clé d'API (D17),
   l'agent qui pilote le navigateur (D18), la sauvegarde et la restauration (D22), la remise à zéro ? Contrôle le
   **PDF aplati**, pas seulement le `.tex` :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}envois? du serveur MCP.{80}"`,
   idem `user-manual.pdf` ; `grep -rnE "ajoute deux tables|en ajoute deux" CHANGELOG.md docs/manual/fr/*.tex` doit
   rendre **0**. Le PDF est-il celui du `.tex` du dernier commit (date, version de la page de titre inchangée — la macro
   de version ne bouge qu'à la release) ? Le PDF de la brochure n'a-t-il **pas** été recommité (source inchangée) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`features/mcp-sends/McpSendDetail.svelte`** — l'état (`recu`, `valeur`, `enEdition`, `envoiEnCours`,
   `erreur`, `dialogueRefus`) : une décision peut-elle partir deux fois (double clic, Entrée dans le dialogue) ? Un
   rechargement (`charger`) pendant une décision en vol ? La réponse d'une décision **remplace**-t-elle bien la valeur
   affichée (`poser`) ; une erreur la laisse-t-elle intacte ? `readonly` d'un envoi décidé ; le dialogue de refus et sa
   garde pendant l'envoi ; le numéro illisible dans l'adresse.
2. **`mcp-sends.api.ts`** — les trois fabriques de corps sont-elles les **seules** à construire un corps ? Un appel
   `decideMcpSend` ailleurs ? `apiClient.post` rejoue-t-il un `POST` (lis `shared/utils/api-client.ts`, `maxAttempts`) ?
3. **`mcp-sends-format.ts`** — `octetsUtf8` contre `str::len` de Rust (demi-paire de substitution, U+FFFD) ;
   `dateLocale` (UTC sans suffixe) ; `erreurDeDecision` : chaque code et statut, l'aide aux caractères après un 400 de
   taille.
4. **`McpSendsList.svelte`**, les deux pages, l'entrée de menu (`+layout.svelte`) : l'ordre du serveur gardé ; aucun
   contenu ; le 403 d'un non-administrateur montré (C-32-8-74 — la page ne redirige pas).
5. **Les tests** — un test peut-il passer **à vide** ? En particulier : `envoie_la_valeur_exacte` compare-t-il le corps
   **entier** ; `la_valeur_affichee_est_la_valeur_recue` lit-il `textarea.value` (et non un attribut) ;
   `mcp-sends-i18n.test.ts` : la fonction `ecartsAuCatalogue` a-t-elle un témoin rouge pour chaque branche, le compte
   exact (48) est-il une borne réelle, la lecture de la constante Rust rougirait-elle si la constante changeait ;
   `ecran_sans_html_brut` couvre-t-il les **deux** dossiers, et son témoin rouge est-il sur une source factice ? L'E2E
   compare-t-il la valeur **lue dans la page** à la valeur **reçue par le client MCP**, ou deux copies de la même
   variable ?
6. **Les catalogues** — 48 clés `mcp-sends-*` et `nav-admin-mcp-sends` dans les **quatre** langues ; placeholders
   (`{ $id }`, `{ $n }`, `{ $max }`, `{ $user }`, `{ $date }`, `{ $code }`) identiques d'une langue à l'autre ;
   `mcp-sends-content-too-large` = `mcp-send-content-too-large` dans chaque langue ; une valeur FTL qui contiendrait une
   accolade ou un caractère qui casserait Fluent.

## Lentille E — Edge cases

1. **Les valeurs** : une `textarea` vidée (chaîne vide : elle part « modifiée », `"content": ""`, et le serveur
   l'admet — C-32-8-55) ; exactement 24 576 octets, un de plus ; un texte de 24 576 caractères ASCII contre
   accentués ; `\r\n` collé (le navigateur normalise-t-il la valeur d'une `textarea` ? que voit le serveur ?) ; un
   U+FE0F, un U+200D, un U+202E collés ; une suite `\u{202E}` tapée en toutes lettres (admise : c'est du texte).
2. **Les états** : envoi en attente, validé tel quel, validé modifié, refusé, échu (en attente et validé : que montre
   le détail d'un envoi validé puis échu ?), contenu `null` dans un état inattendu ; une clé révoquée ; un numéro
   inexistant, d'une autre société (404), illisible.
3. **Les courses** : deux onglets décident le même envoi (409 `ILLEGAL_STATE_TRANSITION`) ; version périmée
   (`OPTIMISTIC_LOCK_CONFLICT`) ; échéance passée pendant la lecture ; *Recharger* après un 409 — l'état d'édition et la
   valeur modifiée sont-ils perdus sans le dire ? ; une remise à zéro simultanée (P3-R6, C-32-8-58, désormais au
   `docs/MULTI-TENANT-SCOPING-PATTERNS.md`) — l'ordre des verrous écrit est-il celui du code
   (`repositories/mcp_sends.rs::decider_in_tx`, `deposer_in_tx`, `services/poignees.rs`, `services/envois_mcp.rs`) ?
4. **Les rôles** : un Comptable, une Consultation atteignent la page par l'adresse (403 montré) ; l'entrée de menu
   `adminOnly` ; un PAT, une clé MCP sur les routes (refusés par le serveur — non à rejouer, à vérifier au manuel).
5. **L'affichage** : un contenu de 24 Kio sur une ligne ; des lettres de droite à gauche visibles (`dir="ltr"` n'y
   peut rien — angle mort écrit ?) ; les dates à l'heure locale contre l'heure UTC du journal d'audit et de l'export.

## Lentille A — Acceptance (la fiche contre le code)

1. **AC-C1 à AC-C11** : chacun est-il tenu **au code**, `grep -nF` à l'appui ? Les écarts déclarés (C-32-8-70 à
   C-32-8-77) sont-ils fondés : la section sous *Sécurité* en attendant la 32-7 ; la garde des replis dans le dossier
   de l'écran plutôt qu'une extension de G13 ; le compteur non bloquant ; l'aide aux sélecteurs de variante ; la page
   sans redirection ; les dates locales ; l'E2E et la capture piégée par `kesh_version` et SQL ?
2. **Les points de l'orchestrateur** : (1) le compteur en octets UTF-8 contre 24 576, même message que le serveur,
   expliqué ; (2) le contrat `"content": null` figé par un test du corps ; (3) le refus de `⚠️` expliqué à l'écran et au
   manuel ; (4) les replis rattachés à une garde réelle ou l'angle mort écrit avec #758 ; (5) l'ordre des verrous au
   document des motifs de portée ; (6) les manuels (`:2249` « deux tables », AC-C11, la note du journal d'audit de la
   32-6b), PDF régénérés, seuls ceux dont la source a changé ; (7) le CHANGELOG à `:16-17` ; (8) les captures.
3. **Les mutations MC1 à MC6** : chacune est-elle tuée **par le test que la fiche nomme** (`328c-mutations.log`,
   `328c-mutations-resultats.json`) ? Sa forme est-elle celle de la fiche, ou une forme **équivalente en effet** (MC3 :
   `{@html}` posé sur le résumé, le contenu n'étant qu'une valeur de champ) ? La seconde cible de MC1 (E2E (1)) n'a pas
   été jouée : est-ce déclaré ? Les fichiers mutés étant neufs (non suivis avant le commit `3ae60eb3`), la restauration
   par copie et `cmp` est-elle vérifiée, et le diff résiduel nul ?
4. **Le Dev Agent Record et le Change Log** : chaque décompte (37 tests Vitest neufs, 1939 → 1976 ; 2 tests E2E ;
   `sitesTotal` 2334 → 2393 ; 48 clés) se recompte-t-il depuis la source, avec son périmètre ? Les gates déclarés
   ont-ils tourné au **dernier commit de code** ? Les échecs E2E sont-ils exactement ceux de la KF-029
   (`docs/testing.md` § *Les échecs attendus*) ?
5. **`sprint-status.yaml`** : la clé `32-8c-ecran-envois` est-elle à `review`, avec un commentaire qui ne dit que ce qui
   a tourné ? Le **README** (feuille de route) et `website/` ont-ils à suivre (`CLAUDE.md` § *Synchroniser*) ?

## Le rapport

Un fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/328c-review-p1-B.md`, `…-E.md`, `…-A.md`. Chacun
contient :

- **les axes exercés et ceux qui ne l'ont pas été**, nommément (les trois axes obligatoires compris) ;
- les findings, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le fichier et la ligne, la preuve
  (sortie `grep -nF` ou `sed -n` copiée, ou la capture nommée), et la correction proposée ;
- le décompte final par sévérité.
