# Epic 32 — registre des choix autonomes de la Story 32-1 (défauts courts, clés 32-1a, 32-1b, 32-1c, 32-1d)

Registre propre à la story (C-32-1-1 à C-32-1-38 ; C-32-1-20 à 28 de la validation P1, C-32-1-29 à 36 de la validation P2, C-32-1-37 du découpage de la 32-1b, C-32-1-38 de la validation P3), à verser **par union** dans `epic-32-choix-autonomes.md` au merge
(`docs/agents-orchestration.md` § 1). Fiche : `32-1-defauts-courts.md`. Clé courte : `321`.
Rédigé le 2026-10-24 (Opus 5.5), worktree `kesh-32-1`, branche `story/32-1` sur `chore/kickoff-epic-32` (`6f0cee00`).

---

### C-32-1-1 — Une fiche, trois clés, un registre

- **Contexte** : C-32-0-3 fixe une fiche pour trois clés (32-1a, 32-1b, 32-1c), comme la 31-1.
- **Retenu** : `32-1-defauts-courts.md`, trois parties A, B, C, numérotation des AC propre à chaque partie (`AC-a1`,
  `AC-b1`, `AC-c1`…) pour qu'une revue d'une clé ne lise que sa partie ; un seul registre (`-321`) ; un seul prompt de
  validation P1, qui couvre les trois parties.
- **Écartées** : trois fiches (trois fois l'en-tête, l'origine, les décomptes) ; une numérotation continue (une revue de
  la 32-1b citerait des AC de la 32-1a).
- **Réversible** : oui.

### C-32-1-2 — La « M4 de la 31-5 » reste équivalente ; c'est la mutation du `COALESCE` qui doit être tuée

- **Contexte** : le fichier d'epic écrit pour la 32-1a « les mutations M4 et M9 de la 31-5 sont **tuées** ». Or la M4 de
  la 31-5 (`31-5-defauts-courts.md`, tableau des mutations) est `if invoice_settings_created` → `if true` dans
  `final_body` (`crates/kesh-seed/src/lib.rs`), déclarée **équivalente** avec sa raison : à l'étape 2 la ligne de
  réglages est toujours neuve, et `COALESCE` n'écraserait rien. Le constat E1 de #716 dit autre chose : la mutation qui
  remplace `COALESCE(col, ?)` par `?` dans `designate_vat_accounts_in_tx`
  (`crates/kesh-db/src/repositories/company_invoice_settings.rs:1321-1323`) **survivrait** — « (M4 équivalente) »
  désigne le motif, non la même mutation. Un test de dépôt qui pose une valeur avant l'appel tue la mutation du
  `COALESCE` ; il ne tue pas la M4 de la 31-5, qui ne passe pas par une ligne préexistante.
- **Retenu** : la 32-1a tue **M-a1** (le `COALESCE` d'une colonne remplacé par `?`) et **M-a2** (`latestLaterClosed`
  sans son filtre `status === 'Closed'`, la M9 de la revue de la 31-5) ; la M4 de la 31-5 reste **équivalente**, et la
  fiche le dit. Prémisse du fichier d'epic corrigée (à signaler à l'orchestrateur).
- **Écartées** : chercher un test qui « tue » la M4 de la 31-5 (il faudrait une ligne de réglages préexistante dans
  `seed_demo`, ce que la démonstration ne produit pas : un tel test prouverait un cas qui n'existe pas).
- **Amendé (validation P1, 2026-10-24)** : « M-a1 » devient **trois** mutations, une par colonne (M-a1a, M-a1b, M-a1c), et le test pose les trois colonnes — C-32-1-21.
- **Réversible** : oui.

### C-32-1-3 — B3 : un test d'existence, pas de repli Rust, pour `invoice-qr-message-prefix`

- **Contexte** : `i18n.format(&locale, "invoice-qr-message-prefix", None)` (`invoice_pdf_service.rs:240`) rend la clé
  brute si elle manque dans les quatre catalogues (`crates/kesh-i18n/src/loader.rs:109-125`), et cette clé brute entrerait
  dans la communication de la QR.
- **Retenu** : un test unitaire dans `invoice_pdf_service.rs` (`mod tests`, à côté de
  `reminder_fees_note_fits_its_width_in_all_four_locales`, qui charge déjà les catalogues réels) : pour chacune des
  quatre locales, la valeur n'est ni vide ni la clé ; pour `de-CH`, `it-CH`, `en-CH`, elle diffère de `fr-CH`.
  **Aucun repli Rust** : un repli rendrait silencieux le retrait de la clé, exactement ce qu'on veut voir rougir.
  `kesh-i18n` n'est donc pas touché.
- **Écartées** : un repli `"Facture"` (masque le défaut au lieu de le signaler, et en français dans une QR allemande) ;
  l'ajout de la clé à G9 (G9 compare des replis existants, il n'en crée pas).
- **Réversible** : oui.

### C-32-1-4 — E-9 (1) : une sonde dans `TestEnv` ; la distinction création / lecture du dossier est équivalente

- **Contexte** : `dossier_auto_inutilisable` (`admin_auto_backup_e2e.rs:684`) n'assert que `failed`. Établi au code
  (`auto_backup.rs:589-600`, dans `ecrire`, l. 577) : deux gardes successives (création `DirBuilder`, puis `read_dir`) rendent
  `failed` ; tout chemin où la création échoue (fichier à la place du dossier, parent non inscriptible) fait aussi
  échouer `read_dir`. Ignorer l'erreur de création est donc une mutation **équivalente** quant au statut.
- **Retenu** : `TestEnv` compte ses appels à `free_bytes` et `wrap_writer` ; le test assert **zéro** appel à chacun —
  le tour s'est arrêté à une garde du dossier, avant la mesure d'espace et avant toute écriture. Mutation **M-a3** :
  neutraliser les **deux** `return` des gardes du dossier (le tour continue jusqu'à `write_backup_file`, qui échoue
  aussi en `failed`) → la sonde rougit. Le doc-comment du test dit l'équivalence de l'ordre création / lecture.
- **Écartées** : capturer le journal (`tracing`) pour lire le message de la garde (aucune capture n'existe dans les tests
  de `kesh-api`, et ce serait la seule) ; réécrire `ecrire` pour rendre l'étape qui échoue (une modification de
  production pour un test).
- **Amendé (validation P1, 2026-10-24)** : `wrap_writer` n'est plus compté : sur ce montage, `write_backup_file` échoue avant `remplir`, le compteur serait à zéro avec ou sans le défaut ; M-a3 est tuée par `free_bytes` seul — C-32-1-25.
- **Réversible** : oui.

### C-32-1-5 — E-9 (2) : la garde lexicale de l'export reste la seule, et le dit

- **Contexte** : le garde de `full_export` (`routes/admin.rs:77-80`) est déplacé dans la fermeture `remplir` pour vivre
  dans le `spawn_blocking` « quel que soit le sort du client ». Sans ce déplacement (M-c24 de la 31-1c), le garde vit
  jusqu'à la sortie du handler : la différence ne s'observe que si le **client part pendant l'assemblage** (le handler
  est abandonné, le verrou rendu, le `spawn_blocking` écrit encore). Un test comportemental exige un crochet de
  synchronisation dans l'assemblage — conception, non mécanique.
- **Retenu** : le doc-comment de `gardes_possedes` (`auto_backup.rs:1190-1201`) dit pourquoi la garde lexicale est la
  seule (propriété observable au seul départ du client) et que c'est un angle mort assumé. Aucun test neuf.
- **Écartées** : un test qui abandonne une requête d'export en vol (dépend du temps d'assemblage, donc flaky) ; un
  crochet de test dans `build_keshbackup` (production modifiée pour un test, hors du périmètre court).
- **Réversible** : oui (le crochet peut venir avec E-1, qui touche le même verrou).

### C-32-1-6 — E-9 (3) : `code()` lit les commentaires de bloc, `argument_de` les littéraux ; `corps()` documenté

- **Contexte** : `code()` (`auto_backup.rs:1121`) n'écarte que les lignes `//` ; `argument_de` (`:1159`) apparie les
  parenthèses sans voir les chaînes ; `corps()` (`:1182`) coupe à la première ligne `}` en colonne 0.
- **Retenu** : `code()` retire aussi les commentaires `/* … */` (imbriqués, comme en Rust) ; `argument_de` ignore les
  parenthèses dans `"…"` (échappements compris) et dans les littéraux `'('` et `')'` ; `corps()` garde sa règle, qui
  repose sur rustfmt (un item de tête se ferme par `}` en colonne 0), et la dit, avec une assertion de montage. Chaque
  helper reçoit son assertion de montage (cas qui rougit avant le correctif).
- **Écartées** : un analyseur Rust (`syn`) pour trois gardes lexicales (une dépendance pour des tests) ; laisser les
  helpers tels quels (le constat est réel : un `/* auto_backup::spawn(x) */` dans `main.rs` compterait).
- **Amendé (validation P1, 2026-10-24)** : le retrait des `/* … */` passe par un **balayeur** qui connaît les chaînes (brutes comprises), les `//` et les littéraux de caractère sans les confondre avec les durées de vie ; assertions de montage tirées de `lib.rs:1186` — C-32-1-20.
- *Annotation de la validation P3 (2026-10-24)* : ce choix ne s'applique plus — E-9 (3) est **sorti de la story** et
  renvoyé à #727 ; la 32-1a n'écrit que la limite des helpers à leurs doc-comments (C-32-1-38).
- **Réversible** : oui.

### C-32-1-7 — E7 : le doc-comment de `kesh-db` seul ; `kesh-seed` n'est pas touché

- **Contexte** : E7 vise le doc-comment de `lock_vat_role_accounts_in_tx`, qui décrit l'ordre des trois verrous de TVA
  sans dire que la transaction complète de `final_body` (1171, 2200, 2206, puis les rôles de
  `insert_with_defaults_in_tx` : `AccountRole::Receivable`, `DefaultRevenue`, `Payable`, trois lectures `FOR UPDATE`,
  `company_invoice_settings.rs:1074-1105`) n'est pas croissante au plan standard : le compte
  `Receivable` (1100) et le compte `Payable` (2000) y viennent **après** 2206 — à relire au plan livré par le
  développement, qui cite les identifiants constatés. Le doc-comment de `final_body`
  (`crates/kesh-seed/src/lib.rs:364-371`) écrit déjà la séquence `accounts (TVA, puis rôles de
  insert_with_defaults_in_tx)` ; il est juste.
- **Retenu** : une phrase de plus au seul doc-comment de `lock_vat_role_accounts_in_tx` : la séquence complète de la
  transaction n'est pas croissante (les rôles d'`insert_with_defaults_in_tx` viennent après, dont deux de numéros plus
  bas au plan standard), et le rejeu de `seed_demo` sur interblocage (`is_seed_retryable`) en est le filet. La 32-1a reste
  à trois modules.
- **Écartée** : retoucher aussi `final_body` (un cinquième site pour une phrase déjà juste).
- **Amendé (validation P1, 2026-10-24)** : la séquence n'est pas « les trois rôles » : `insert_with_defaults_in_tx` verrouille aussi, par `chart_designated_accounts`, jusqu'à quatre comptes désignés par le plan — C-32-1-23.
- **Réversible** : oui.

### C-32-1-8 — E-6 : une saisie non enregistrée survit à « Sauvegarder maintenant » ; « Recharger » la remplace

- **Contexte** : `runNow` (`AutoBackupSection.svelte:139-153`) passe la réponse à `apply` (`:63-69`), qui réécrit
  `enabled`, `intervalHours`, `retentionCount`. Les écritures de la tâche (`record_*`) ne touchent jamais `version`
  (`crates/kesh-db/src/repositories/backup_schedule.rs:9-15`) : la version lue reste valable après un run.
- **Retenu** : une fonction `applyState(value)` — l'état (`schedule`, état partagé) toujours, les trois champs du
  formulaire **seulement s'ils égalent encore le réglage lu** (formulaire non modifié). `runNow` et la relecture de
  l'AC-b2 l'emploient ; `loadSchedule` (montage, « Recharger » après un 409) et `save` gardent `apply` (remplacement
  complet, voulu).
- **Écartées** : un avertissement avant « Sauvegarder maintenant » (un dialogue de plus pour un geste fréquent) ;
  enregistrer d'office la saisie avant le run (une écriture que l'utilisateur n'a pas demandée).
- **Amendé (validation P2, 2026-10-24)** : la saisie gardée l'est **avec la version sur laquelle elle a été faite**
  (`formBase`, C-32-1-29) ; « Recharger » remplace la saisie aussi dans le cas concurrent, par le 409 qui joue désormais.
- **Réversible** : oui.

### C-32-1-9 — E-7 : tout échec de « Sauvegarder maintenant » autre qu'un 409 `BACKUP_IN_PROGRESS` relit l'état

- **Contexte** : le délai client de `runBackupNow` est `LONG_OPERATION_TIMEOUT_MS` (60 min,
  `frontend/src/lib/shared/utils/api-client.ts:53`) ; mais un proxy coupe bien avant (Nginx `proxy_read_timeout 300s`,
  manuel § proxy), et rend un 502/504 ; un `record_success` en erreur rend 500 alors que l'archive est écrite (E-2).
  Dans tous ces cas, l'écran dit « n'a pas pu être lancée » sans relire, alors que la sauvegarde a pu être écrite.
- **Retenu** : sur tout échec **autre que** 409 `BACKUP_IN_PROGRESS` (dont le message serveur reste affiché tel quel),
  l'écran affiche un message neuf — la réponse n'est pas parvenue, la sauvegarde a pu être écrite, l'état et la liste
  ont été relus — et relit l'état (`applyState`, C-32-1-8) et la liste. Clé neuve `admin-backup-auto-run-unknown`.
- **Écartées** : distinguer `TIMEOUT` / `NETWORK_ERROR` des 5xx (le résultat est inconnu dans les deux cas) ; sonder
  l'état jusqu'à voir changer `lastAttemptAt` (une boucle pour un cas rare).
- **Amendé (validation P1, 2026-10-24)** : « tout échec autre que 409 » est **remplacé** : le message « pas parvenue » est réservé à l'issue inconnue (prédicat d'`issueInconnue` de l'import, extrait) ; les autres 5xx gardent le message du serveur et relisent ; les 4xx gardent le message du serveur sans relire — C-32-1-22. L'écartée « distinguer `TIMEOUT` / `NETWORK_ERROR` des 5xx » est reprise en partie.
- **Réversible** : oui.

### C-32-1-10 — E-8 : un échec de sondage garde l'état connu ; une réponse tardive est ignorée

- **Contexte** : `refreshBackupScheduleState` (`backup-schedule.svelte.ts:30-36`) pose `null` sur tout échec, et une
  réponse lancée avant `setBackupScheduleState` peut l'écraser. La fiche de la 31-1c (AC31) dit « un échec de cette
  lecture n'affiche aucun bandeau » : vrai d'une **première** lecture (l'état est `null`) ; l'éteindre sur un échec
  passager n'en découle pas.
- **Retenu** : un échec de lecture **laisse** l'état tel qu'il est (s'il est `null`, aucun bandeau : le test « lecture en
  échec : absent » reste vrai) ; un compteur de séquence, incrémenté par `setBackupScheduleState` et par chaque
  lecture, fait ignorer toute réponse dont la séquence n'est plus la dernière (motif du jeton de requête de C-29-1-37,
  `fiscal-years`). Le doc-comment du composant et le manuel (§ *Le bandeau de l'application*) le disent.
- **Écartées** : garder l'état connu avec une date d'expiration (une horloge du navigateur, que le module s'interdit) ;
  un bandeau « état inconnu » (bruit sur chaque coupure réseau).
- **Réversible** : oui.

### C-32-1-11 — L-B5 : le serveur dit qu'une réussite est datée du futur, le bandeau le dit en clair

- **Contexte** : `is_overdue` est vrai pour une réussite postérieure à `now` (`auto_backup.rs:221-226`, voulu : horloge
  revenue en arrière) ; le bandeau affiche alors « Aucune sauvegarde automatique réussie depuis le <date future> »
  (`BackupScheduleBanner.svelte:33`, clé `admin-backup-banner-overdue`). Le navigateur ne compare aucune date à sa propre
  horloge (doctrine du module, en-tête de `backup-schedule.api.ts`) : la distinction doit venir du serveur.
- **Retenu** : une fonction pure `is_success_in_future(schedule, now)` dans `auto_backup.rs` (vrai si
  `last_success_at` existe et que `passe()` l'écarte), un champ `lastSuccessInFuture: bool` dans
  `BackupScheduleResponse` (`routes/backup_schedule.rs`) ; le bandeau, dans ce cas, affiche la clé neuve
  `admin-backup-banner-future` (la dernière réussite est datée du { $date }, après l'heure du serveur : son horloge a
  reculé). `bannerKind` ne change pas (le bandeau reste `overdue`, `data-kind="overdue"`).
- **Écartées** : masquer la date (le libellé « depuis le » sans date est absurde aussi) ; ne pas allumer le bandeau (le
  manuel, § *Horloge du serveur déréglée*, dit qu'il s'allume, et c'est utile : l'horloge est fausse) ; comparer dans le
  navigateur (contraire à la doctrine du module).
- **Amendé (validation P1, 2026-10-24)** : `is_success_in_future` exige aussi `enabled`, comme `is_overdue` — C-32-1-24.
- **Réversible** : oui (un champ additionnel de réponse).

### C-32-1-12 — L-B6 : la mention garde les Mio de la variable et ajoute l'équivalent décimal

- **Contexte** : la ligne affiche sa taille en unités décimales (`formatBackupSize`, `backup-format.ts:14-28`) et la
  mention « limite N Mio » compare à `importMaxMib × 1 048 576` (`backup-schedule.api.ts:130`). Les Mio sont l'unité de
  `KESH_ADMIN_IMPORT_MAX_MB`, celle que l'administrateur relève.
- **Retenu** : `admin-backup-server-too-large` prend une variable de plus, `{ $size }` = la limite formatée par
  `formatBackupSize` : « trop grande pour être restaurée par l'écran (limite { $mib } Mio, soit { $size }) » ; quatre
  locales ; le manuel (§ *Les sauvegardes du serveur*) cite la nouvelle forme.
- **Écartées** : afficher les tailles en unités binaires (`Intl.NumberFormat` n'a pas d'unité `mebibyte`, et toutes les
  autres tailles de l'écran changeraient) ; n'afficher que l'équivalent décimal (l'administrateur ne saurait plus quelle
  valeur de la variable relever).
- **Réversible** : oui.

### C-32-1-13 — L-B7 : le refus traduit, prouvé dans `i18n_e2e.rs`

- **Contexte** : `update_backup_schedule` rend `AppError::Validation("Réglage de sauvegarde hors domaine : …".into())`
  (`routes/backup_schedule.rs:133-137`), en français dans les quatre langues. Le refus du `PUT` est servi par un bloc
  `admin_routes` ; `i18n_e2e.rs` monte déjà l'application dans une locale donnée (`spawn_app_with_locale`, l. 65) avec un
  administrateur (`ensure_admin_user`) et son propre verrou de l'état i18n global (l. 25-32).
- **Retenu** : `crate::errors::t("admin-backup-schedule-error-out-of-domain", "<valeur fr-CH>")`, la clé dans les quatre
  catalogues, une entrée de plus à la table de G9 (`textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`), et un
  test neuf dans `i18n_e2e.rs` : en `de-CH`, `PUT /api/v1/admin/backup-schedule` avec `intervalHours: 7` rend 400, un
  message égal au rendu `de-CH` de la clé et différent du rendu `fr-CH`.
- **Écartées** : le test dans `admin_auto_backup_e2e.rs` (ses autres tests supposent le français ; un `init_error_i18n`
  en `de-CH` y ferait dériver leurs messages sous `cargo test`, processus partagé) ; dupliquer `spawn_app_de` de
  `audit_log_e2e.rs` (règle DRY, et c'est le constat B2).
- **Réversible** : oui.

### C-32-1-14 — E-1 et E-2 : une seule issue, deux parties

- **Contexte** : E-1 (verrou tenu sans limite par une écriture figée ; aucun bandeau sans réussite antérieure) et E-2
  (`record_success` en erreur après l'écriture) demandent chacun une conception — un délai et ce qu'il fait d'une
  écriture en cours ; le chemin d'erreur d'une archive écrite mais non enregistrée (rétention, réponse, reprise). Établi
  au code : `run_once` prend le verrou en `try_lock_owned` (`auto_backup.rs:669`) et le garde jusqu'à la fin ; export et
  import l'attendent en `lock_owned().await` (`routes/admin.rs:77`, `:213`) ; `is_overdue` est faux sans
  `last_success_at` (`auto_backup.rs:221-226`) ; `record_success` (`:713`) en `Err` remonte par `?` (aucune rétention,
  500 au `POST …/run`, `last_attempt_at` déjà posé : reprise dans une heure, seconde archive).
- **Retenu** : une issue « robustesse du tour de sauvegarde automatique », deux parties, texte prêt dans la fiche
  (*Issue à ouvrir*, devenue *Issues ouvertes*) ; l'orchestrateur peut la scinder.
- **Écartées** : les intégrer à la 32-1b (conception d'un délai et d'un chemin d'erreur, non mécanique — C-32-0-3) ; deux
  issues (même module, même verrou, même revue probable).
- **Amendé (validation P1, 2026-10-24)** : l'issue est ouverte : **#725** (`bug`, `P3`), en une seule issue à deux parties.
- **Réversible** : oui.

### C-32-1-15 — 32-1c : une phrase se corrige ici ; un geste se renvoie à Guy

- **Contexte** : le fichier d'epic : « la fiche ne rejoue pas la procédure, elle corrige des phrases ; toute correction
  qui changerait un geste se renvoie à Guy ». Les deux choix de procédure de #714 sont tranchés (commentaire de Guy du
  2026-10-24, versés en v0.15.0 : vérifié au manuel, `docs/manual/fr/admin-manual.tex:1791` — l'écart est un arrêt — et
  `:1906` — le `pull` en échec, le troisième listing).
- **Retenu** : la règle de partage, décidable : **la 32-1c** corrige ce qui est faux ou incomplet dans ce que la
  procédure **dit** — une cause nommée, une sortie attendue, une source d'écriture à éviter, une limite d'un contrôle —
  sans ajouter, retirer ni changer **aucune commande** ni **aucune consigne d'agir** ; **tout le reste** (une commande
  changée, une consigne neuve : refaire, refuser, effacer, remplacer) va dans une issue plus étroite, pour Guy (texte
  prêt dans la fiche). Une fois la 32-1c fusionnée et l'issue ouverte, #714 n'a plus de constat propre.
- **Écartées** : tout corriger dans la 32-1c (la procédure n'a été jouée qu'une fois, dans une rédaction antérieure :
  une commande changée sans rejeu est une procédure non éprouvée de plus) ; tout renvoyer à Guy (des phrases fausses
  resteraient au manuel tant qu'il n'a pas tranché).
- **Amendé (validation P1, 2026-10-24)** : l'issue des gestes est ouverte : **#726** (`documentation`, `P3`, sans `bug`).
- **Réversible** : oui.

### C-32-1-16 — Le glyphe ⚠️ remplacé par une macro, dans les deux manuels FR (défaut voisin mécanique)

- **Contexte** : #714 relève que ⚠️ ne s'imprime pas dans le PDF du manuel d'administration (16 lignes du `.tex`).
  Vérifié : `pdftotext docs/manual/fr/admin-manual.pdf - | grep -c "⚠"` rend 0 ; la police de corps (TeX Gyre Heros,
  `docs/manual/shared/kesh-style.sty:95`) n'a pas ce glyphe. Le manuel utilisateur porte le même défaut :
  `grep -c "⚠" docs/manual/fr/user-manual.tex` rend **9**, son PDF 0. La brochure : 0. `kesh-style.sty:37` fournit déjà
  `\faExclamationTriangle` (repli `\textbf{!}`), employé par l'encadré `keshwarning`.
- **Retenu** : une macro `\keshAttention` dans `kesh-style.sty`, définie sur `\faExclamationTriangle`, et les 16 + 9
  occurrences remplacées (correction mécanique, même cause, intégrée comme défaut voisin) ; contrôle : `grep -c "⚠"` sur
  les deux `.tex` rend 0.
- **Écartées** : une police de repli à emoji (`fontspec` + police couleur : dépendance de la chaîne TeX du poste et de la
  CI) ; le seul manuel d'administration (le même défaut resterait, mécanique, dans l'autre).
- **Amendé (validation P1, 2026-10-24)** : « 16 + 9 occurrences » étaient des **lignes** : 17 + 9 = **26** occurrences (la l. 2923 d'`admin-manual.tex` en porte deux), toutes suivies du sélecteur U+FE0F, remplacé avec elles ; emploi `\keshAttention\ ` ; la brochure se régénère aussi — C-32-1-26.
- **Amendé (validation P2, 2026-10-24)** : le contrôle `pdftotext … | grep -c "⚠"` rend 0 avant comme après — il ne
  prouve rien ; et « la brochure : 0 » est vrai de `⚠`, faux de la classe. La classe se recense par son **symptôme**
  (U+FFFD de `pdftotext`) : 26 / 9 / 16 ; elle est traitée entière par la 32-1c (C-32-1-31).
- **Réversible** : oui.

### C-32-1-17 — Ordre de fusion : 32-1c, puis 32-1b ; le second régénère le PDF

- **Contexte** : la 32-1b et la 32-1c modifient toutes deux `docs/manual/fr/admin-manual.tex` (sections distinctes,
  fusion textuelle sans conflit) et son PDF (binaire : conflit à toute fusion parallèle).
- **Retenu** : la 32-1c (doc seule, plus courte) se fusionne d'abord ; la 32-1b se met à jour sur `main` avant sa revue
  finale et **régénère** le PDF (`make fr`) sur le `.tex` fusionné. La 32-1a ne touche pas le manuel.
- **Écartée** : une PR unique pour les trois clés (trois revues dans une PR, contraire à C-32-0-3).
- **Amendé (validation P1, 2026-10-24)** : la 32-1a et la 32-1b partagent aussi trois fichiers de code (hunks distincts) ; la clé fusionnée en second se rebase et repasse son gate complet ; `closes #711` porté par celle des deux qui fusionne en dernier — C-32-1-27.
- **Amendé (découpage, 2026-10-24)** : la 32-1d (serveur) ne touche pas le manuel ; l'ordre 32-1c → 32-1b reste, et
  s'y ajoute 32-1d → 32-1b (le bandeau lit le drapeau servi) — C-32-1-37.
- **Réversible** : oui (l'ordre inverse marche, en inversant la régénération).

### C-32-1-18 — Gates : complet pour 32-1a et 32-1b, E2E complet au dernier commit de code de chacune ; 32-1c ciblé

- **Contexte** : la 32-1a touche un test de `kesh-db` (exception `kesh-db` du `CLAUDE.md`) ; la 32-1b change du code
  servi (frontend, route) ; la 32-1c ne touche que le manuel et deux fiches — mais trois gardes de `kesh-api` lisent le
  manuel (`textes_coherents.rs`, `configuration_transmise.rs`).
- **Retenu** : 32-1a et 32-1b : gate complet (`scripts/test-fast.sh` sous verrou, base remise à zéro d'abord), les quatre
  gates frontend, **E2E complet au dernier commit de code** (la 32-1a ne change aucune ligne servie, mais le `CLAUDE.md`
  ne fait pas d'exception pour un commit de tests ; l'E2E est le moins cher des arguments). 32-1c : gate ciblé
  `binary(textes_coherents)` et `binary(configuration_transmise)`, `make fr`, contrôle aplati du PDF ; pas d'E2E (aucun
  code).
- **Écartée** : pas d'E2E pour la 32-1a (défendable, mais une revue le contesterait ; le coût d'une contestation est
  supérieur au coût du run).
- **Amendé (découpage, 2026-10-24)** : la 32-1d reçoit le même régime que la 32-1b — gate complet avant le push (ciblage
  permis en boucle, pas de `kesh-db`), les quatre gates frontend (les catalogues changent), E2E complet au dernier commit
  de code (la réponse servie change) ; l'E2E final de la 32-1b se joue après la fusion de la 32-1d — C-32-1-37.
- **Réversible** : oui.

### C-32-1-19 — Prémisses des issues corrigées au code

- **Contexte** : chaque constat a été relu au site cité, le 2026-10-24, sur `6f0cee00`.
- **Retenu** : les corrections, écrites dans la fiche (§ *Constats établis au code*) et à signaler sur les issues par
  l'orchestrateur : #711 E-7 cite `backup-schedule.api.ts:111` — c'est `downloadServerBackup` ; `runBackupNow` est aux
  l. 95-100 ; #711 L-B3 cite `auto_backup.rs:490-491` — la fonction du seuil est `seuil_d_espace`, l. 496-500 ; #711
  E-9 (1) — « `failed` est aussi le statut de repli de `build_keshbackup` en `Err` » est juste, mais la seule
  distinction **observable** est l'absence de mesure d'espace et d'écriture (C-32-1-4) ; #714 « laissez le dossier vide
  laisse un reste qui bloque une reprise le même jour » — faux à la lettre : la reprise se fait dans un **nouveau**
  dossier (suffixe), que le `mkdir` du contrôle 3 crée sans conflit ; le reste est un dossier vide, inoffensif, que
  `/var/tmp` garde trente jours ; #716 E6 cite `textes_coherents.rs:276,295` — la regex est l. 277-278, son emploi
  l. 296 ; le fichier d'epic, pour la 32-1a, « M4 de la 31-5 tuée » (C-32-1-2).
- **Réversible** : sans objet (relevé).

---

*Choix C-32-1-20 à C-32-1-28 : remédiation de la validation P1 (2026-10-24, Opus 5.5), sur les rapports
`kesh-gate-logs/321-validate-p1-R.md` et `321-validate-p1-F.md`.*

### C-32-1-20 — Un balayeur lexical partagé, qui connaît les chaînes avant les commentaires (R1 = F1)

- **Contexte** : `lib.rs:1186` porte `/*` **dans une chaîne** (`"KESH_TEST_MODE=true — /api/v1/_test/* exposé …"`) ;
  le fichier n'a aucun `*/` ni `#[cfg(test)]`. Un retrait de `/* … */` qui ignore les chaînes y ouvre un commentaire
  jamais fermé : la fin de `build_router` (dont `.with_state(state)`, l. 1200) sort de la garde « zéro lancement dans
  `lib.rs` », qui reste **verte**. Les deux assertions d'école de la fiche passaient avec ce défaut. Les fichiers lus
  portent aussi des chaînes brutes (`auto_backup.rs:66`, `:72`) et des durées de vie `&'static` (`auto_backup.rs:258`,
  `routes/backup_schedule.rs:194`), qu'un balayeur naïf prendrait pour des littéraux de caractère.
- **Retenu** : un seul balayeur à états (code, `//`, `/* */` imbriqués, `"…"`, `r#"…"#`, littéraux `'x'` / `'\x'`
  seulement), partagé par `code()` et `argument_de` ; assertions de montage tirées du réel (une chaîne contenant `/*`,
  `code(include_str!("lib.rs"))` contient `.with_state(state)`, une durée de vie) ; mutation M-a7 (retrait naïf) tuée.
- **Écartées** : retirer les blocs ligne à ligne avant ou après le filtre `//` (les deux ordres aveuglent `lib.rs`,
  l. 1179, 1186, 1196) ; `syn` (C-32-1-6).
- **Amendé (validation P2, 2026-10-24)** : le balayeur n'est pas à écrire, il **existe** (`kesh-db/tests/lexique/mod.rs`,
  `decouper`) et se réemploie, étendu aux blocs imbriqués ; M-a7 en deux formes et assertion (3') — C-32-1-32.
- *Annotation de la validation P3 (2026-10-24)* : sans objet — l'AC-a11 est retiré, aucun balayeur n'est écrit ni
  réemployé par la story ; E-9 (3) va à #727 (C-32-1-38).
- **Réversible** : oui.

### C-32-1-21 — E1 : les trois colonnes posées, trois mutations, `UPDATE` après le `commit` (R3 = F2)

- **Contexte** : seule `default_vat_payable_account_id` était posée avant l'appel ; sur `NULL`, `COALESCE(col, ?)` et
  `?` écrivent la même chose : les mutations des deux autres colonnes survivaient.
- **Retenu** : deux tests — les trois colonnes posées à des valeurs légitimes et distinctes des rôles (2000, 1170, 2201
  au plan PME, relues en base), puis le remplissage et l'idempotence ; M-a1a, M-a1b, M-a1c, une par colonne. Montage :
  création de la ligne dans une transaction **commitée**, `UPDATE` directs sur le pool **après** ce `commit` (sinon ils
  attendraient le verrou de la ligne), puis appels dans des transactions commitées.
- **Écartées** : un seul test à trois colonnes posées et un second appel (l'idempotence sur `NULL` d'origine ne serait
  plus exercée) ; l'`UPDATE` dans la même transaction (possible, mais un test qui mêle le montage et l'objet se relit
  moins bien).
- **Réversible** : oui.

### C-32-1-22 — E-7 : « la réponse n'est pas parvenue » réservé à l'issue inconnue (F3 = R6)

- **Contexte** : C-32-1-9 mettait le message neuf sur tout échec autre que 409 : un 401, 403, 423 ou 429 — le serveur a
  répondu, rien n'a été écrit — aurait dit « la sauvegarde a peut-être été écrite ». Le dépôt a un précédent :
  `issueInconnue` de l'import (`AdminRestorePanel.svelte:61-68` : `TIMEOUT`, `NETWORK_ERROR`, 502, 504). Le texte
  promettait aussi « ont été relus » alors que la relecture peut échouer, et l'échec de la relecture de l'état n'avait
  pas de message nommé.
- **Retenu** : trois cas. Issue inconnue (le prédicat d'`issueInconnue`, **extrait** dans `lib/shared/utils/` et
  employé par les deux écrans) → message neuf, qui dit « l'écran relit … : vérifiez-les », et relecture. Autre 5xx
  (500 d'E-2, 503) → message du serveur et relecture (l'archive a pu être écrite). 4xx → message du serveur, sans
  relecture. Un échec de la relecture de l'état pose `loadError` (texte de `loadSchedule`), la liste `listError`.
  Mutation M-b8 (message neuf étendu aux 4xx) tuée par le cas 403.
- **Écartées** : recopier le prédicat dans `AutoBackupSection` (règle DRY) ; le message neuf pour le 500 (le serveur a
  répondu : « pas parvenue » serait faux) ; aucune relecture pour le 500 (c'est le cas où l'archive est écrite, E-2).
- **Réversible** : oui.

### C-32-1-23 — E7 : la séquence des verrous établie au code, non énumérée comme exhaustive (R2)

- **Contexte** : la fiche écrivait « les trois lectures `FOR UPDATE` d'`insert_with_defaults_in_tx` » ; la fonction
  appelle aussi `chart_designated_accounts` (`company_invoice_settings.rs:1118`), qui verrouille par
  `designated_account_id` (`FOR UPDATE`, l. 512) jusqu'à quatre comptes désignés par le plan — arrondi, puis rabais,
  frais bancaires, pertes sur créances (`WriteOffNature::ALL`). Au plan PME : 1171, 2200, 2206, 1100, 3000, 2000, 6940,
  3800, 6900, 3805.
- **Retenu** : le doc-comment dit les deux familles de lectures d'`insert_with_defaults_in_tx`, la séquence au plan PME
  à titre d'exemple, et renvoie aux deux fonctions qui la portent plutôt que d'affirmer une liste close.
- **Écartée** : écrire la liste de dix comme exhaustive (elle dépend du plan, et changera avec lui).
- **Réversible** : oui.

### C-32-1-24 — L-B5 : `is_success_in_future` exige `enabled`, comme `is_overdue` (R5)

- **Contexte** : `is_overdue` (`auto_backup.rs:221-226`) est faux si la sauvegarde est désactivée ; la fonction
  prescrite ne regardait pas `enabled` : désactivée et datée du futur, l'API aurait servi `lastSuccessInFuture: true`
  et `overdue: false`.
- **Retenu** : `enabled &&` ; `lastSuccessInFuture` implique `overdue` ; cas de test « futur et désactivée → faux ».
- **Écartée** : la sémantique sans `enabled` (vraie d'un horodatage, mais deux drapeaux contradictoires à l'API pour
  un bandeau qui ne lit le second que sous le premier).
- **Réversible** : oui.

### C-32-1-25 — E-9 (1) : la sonde compte `free_bytes` seul (R4)

- **Contexte** : sur le montage de `dossier_auto_inutilisable` (`auto` est un fichier), même les deux gardes
  neutralisées, `write_backup_file` échoue à créer `auto/<nom>.partial` (`ENOTDIR`, `routes/admin.rs:747`) avant
  d'appeler `remplir`, donc `wrap_writer` : un compteur de `wrap_writer` vaut zéro avec ou sans le défaut.
- **Retenu** : compter `free_bytes` seul, et dire au doc-comment pourquoi `wrap_writer` ne l'est pas.
- **Écartée** : garder l'assertion inerte (un test qui prouve moins qu'il n'annonce).
- **Réversible** : oui.

### C-32-1-26 — Brochure régénérée et commitée ; CHANGELOG `## [0.16.0] — Non publié`, rubrique `Corrigé` (R11 = F8 = F10)

- **Contexte** : `docs/manual/Makefile` (l. 79, `%.pdf: %.tex shared/kesh-style.sty …`) reconstruit les trois PDF dès
  que le `.sty` change ; la fiche n'en nommait que deux. Le CHANGELOG n'a plus de bloc non publié depuis la 0.15.0.
- **Retenu** : `marketing-brochure.pdf` régénéré **et commité** (un PDF plus ancien que son `.sty` est périmé au sens du
  `Makefile`), avec contrôle que son texte ne change pas ; CHANGELOG : bloc `## [0.16.0] — Non publié` (la forme que
  `scripts/prepare-release.sh` date), créé par la première clé qui fusionne s'il n'existe pas sur `main`, entrées sous
  `### Corrigé`.
- **Écartées** : restaurer la brochure par `git checkout` (le `make` suivant la reconstruirait) ; une section
  `[Unreleased]` en anglais (contraire au fichier).
- **Amendé (validation P2, 2026-10-24)** : le texte de la brochure change en ses 16 signes sans glyphe (C-32-1-31) ;
  le contrôle compare au PDF de la **base de la branche** (`git merge-base origin/main HEAD`), jamais à `HEAD:` (F7) ;
  le bloc du CHANGELOG est créé par la première **des 32-1b et 32-1c** qui fusionne (F6).
- **Amendé (validation P3, 2026-10-24, F3)** : depuis le découpage (C-32-1-37), le bloc est créé par la première **des
  32-1c et 32-1d** qui fusionne — la 32-1b fusionne après les deux et n'est jamais la première ; un seul bloc, le
  conflit textuel au rebase de la seconde étant prévu et résolu par union (fiche, *Pourquoi quatre clés*). Le
  découpage aurait dû porter cet amendement ; il ne l'a pas fait.
- **Réversible** : oui.

### C-32-1-27 — Fichiers communs A/B, ordre de fusion, `closes #711` sur la dernière (R10 = F7)

- **Contexte** : la fiche disait les trois clés « sans fichier de code commun » ; la 32-1a et la 32-1b partagent
  `auto_backup.rs`, `admin_auto_backup_e2e.rs` et `textes_coherents.rs` (hunks distincts). Et `closes #711` n'était
  prévu que sur la 32-1b : fusionnée la première, #711 restait ouverte, E-9 attendant la 32-1a.
- **Retenu** : la seconde des deux à fusionner se rebase sur `main` et repasse son gate complet ; `closes #711` est
  porté par celle qui fusionne **en dernier**, l'autre `refs #711`, vérification après fusion.
- **Écartée** : imposer l'ordre 32-1a puis 32-1b (sans raison technique : les hunks ne se recouvrent pas).
- **Amendé (découpage, 2026-10-24)** : les trois fichiers de code communs passent avec la partie serveur — ils sont
  partagés par la 32-1a et la **32-1d** ; la 32-1b n'en touche plus aucun. `closes #711` va à la dernière fusionnée des
  32-1a, 32-1b, 32-1d ; la 32-1d fusionnant avant la 32-1b, elle porte toujours `refs #711` — C-32-1-37.
- **Réversible** : oui.

### C-32-1-28 — `lastSuccessInFuture` requis côté client, fixtures nommées (R7 = F9)

- **Contexte** : un champ requis fait rougir `npm run check` sur les fixtures typées `BackupSchedule`.
- **Retenu** : champ **requis** (le serveur le sert toujours) ; fixtures complétées : `backup-schedule.api.test.ts:44`,
  `AutoBackupSection.test.ts:45`, `BackupScheduleBanner.test.ts:43`, et `etat()` de `backup-schedule.spec.ts:111-125`.
- **Écartée** : champ optionnel (le type dirait moins que le contrat servi).
- **Amendé (découpage, 2026-10-24)** : le champ est servi par la 32-1d, déclaré requis par la 32-1b ; « requis » n'est
  vrai qu'une fois la 32-1d fusionnée, d'où l'ordre 32-1d → 32-1b — C-32-1-37.
- **Réversible** : oui.

---

*C-32-1-29 à C-32-1-36 : remédiation de la validation P2 (Opus 5.5, 2026-10-24 ; rapports
`kesh-gate-logs/321-validate-p2-R.md`, `321-validate-p2-F.md`).*

### C-32-1-29 — E-6 : la saisie gardée garde sa version ; le 409 joue (F1)

- **Contexte** : `save` envoie `schedule.version` (`AutoBackupSection.svelte:103-118`). L'AC-b1 de la P1 faisait poser
  `schedule` par `applyState` sous une saisie gardée : si un autre administrateur avait enregistré entre la lecture et le
  run (version *n* + 1 dans la réponse), l'enregistrement suivant partait contre *n* + 1 et passait — son réglage
  écrasé sans 409, la rétention suivante appliquée au nombre de la saisie. La Dev Note disait l'inverse.
- **Retenu** : un état `formBase` (le réglage sur lequel la saisie a été faite), posé par `apply`, et par `applyState`
  seulement quand il réécrit le formulaire ; `save` envoie `formBase.version`. Le cas normal ne change pas (`record_*`
  ne touche pas `version`) ; le cas concurrent rend 409 et ouvre « Recharger », comportement existant. Vitest (iii),
  mutation M-b9.
- **Écartées** : `applyState` qui ne garde la saisie que si la version n'a pas bougé (la saisie perdue en silence dans
  le cas concurrent, ce que l'E-6 voulait éviter) ; un avertissement de conflit propre au run (un état d'écran de plus,
  quand le 409 existant dit déjà la chose).
- **Réversible** : oui.

### C-32-1-30 — E-7 : la relecture ne dit pas le résultat d'une sauvegarde en cours ; l'écran le dit (F2)

- **Contexte** : le cas d'E-7 est un proxy qui coupe avant la fin ; la tâche `tokio::spawn`
  (`routes/backup_schedule.rs:152`) continue, et la relecture tombe entre `record_attempt` (qui ne pose que
  `last_attempt_at`) et `record_success`/`record_failure` (qui ne pose aucune date). Le panneau montre alors la date de
  la tentative en cours avec le statut de la précédente (`AutoBackupSection.svelte:339-343`). Rien dans l'état servi ne
  distingue « en cours » de « terminée en échec » : le dire à l'écran demande un champ neuf.
- **Retenu** : formulation honnête — le message neuf dit « peut-être écrite, ou s'écrit encore » et invite à recharger
  la page dans quelques minutes ; la phrase du manuel ne dit plus « où se lit le résultat » et décrit ce que montre une
  tentative en cours. L'affichage, défaut préexistant, va en **commentaire de #725**, dont la partie 1 prévoit déjà un
  signal « tentative en cours » (aucune issue neuve).
- **Écartées** : un champ `running` servi (tiré de l'état de `backup_lock`, que l'export et l'import prennent aussi :
  une conception, celle de #725) ; une relecture différée ou répétée côté client (un minuteur dont la durée ne se
  justifie pas, pour une écriture de durée inconnue).
- **Réversible** : oui.

### C-32-1-31 — Les signes sans glyphe : la classe entière, par son symptôme, dans les trois PDF (F3 = R2)

- **Contexte** : `pdftotext` rend U+FFFD pour chaque caractère sans glyphe : 26 (administration), 9 (utilisateur), 16
  (brochure). Au-delà des 26 `⚠️` : 9 signes de deux scripts d'exemple du manuel d'administration (`✓`, `✗`, `═`,
  l. 1547, 3313, 3318, 3320 ; TeX Gyre Cursor), 15 `\faCheckCircle` (`\textbf{✓}`, `kesh-style.sty:39`) et 1
  `\faExchangeAlt` (`\textbf{⇄}`, l. 42) dans la brochure. Lu dans la table `cmap` des `.otf` de TeX Live : ni Heros ni
  Cursor n'ont U+2713, U+2717, U+21C4, U+2550, U+26A0 ; tous deux ont `+`, `x`, `=`, `→`.
- **Retenu** : même cause, même fichier, geste mécanique : `\faCheckCircle` → `\textbf{+}`, `\faExchangeAlt` →
  `\textbf{→}` (le repli de `\faRocket`), `literate={✓}{{+}}1 {✗}{{x}}1 {═}{{=}}1` au style `kesh` des listings ; aucun
  `.tex` touché pour ces 25 sites. Contrôle décidable : U+FFFD → 0 dans les trois PDF, aux deux bornes. Repli écrit si
  `literate` ne prend pas un caractère multioctet : les 9 signes en ASCII dans les deux scripts (hors
  `sec:repetition-cloture`).
- **Écartées** : une police de repli (DejaVu, présente sur la station mais **hors** de TeX Live : une dépendance neuve
  de la chaîne) ; une issue (le geste est d'une ligne par signe, dans le fichier que la 32-1c touche déjà) ; `amssymb`
  pour `\checkmark` (un paquet de plus pour un signe).
- **Réversible** : oui (une ligne du `.sty` par signe ; le choix du signe de repli est de présentation).

### C-32-1-32 — Le balayeur lexical se réemploie : `lexique::decouper`, étendu aux blocs imbriqués ; M-a7 en deux formes (R1, F9)

- **Contexte** : (R1) M-a7, « un `code()` qui retire `/*…*/` sans connaître les chaînes », était indécidable : sous sa
  forme non gourmande, ni le texte de (3) ni `lib.rs` n'ont de `*/`, rien n'est retiré, (3) et (4) restent vertes, et
  la mutation survit. (Remédiation) En complétant l'inventaire de R4 : `crates/kesh-db/tests/lexique/mod.rs` porte déjà
  un analyseur juste (`decouper` : chaînes ordinaires et brutes, `//`, `/* */` non imbriqués, caractères distingués des
  durées de vie, copie masquée de même longueur, fins de ligne gardées), inclus par `#[path]` dans
  `kesh-api/tests/bank_account_closed_inventory.rs:61`. L'AC-a11 prescrivait un second balayeur : contraire à la règle
  DRY, dans la story même qui inventorie les copies.
- **Retenu** : `#[cfg(test)] #[path = "../../kesh-db/tests/lexique/mod.rs"] mod lexique;` en tête d'`auto_backup.rs`
  (hors du `mod tests` en ligne, dont le chemin se résoudrait sous `src/auto_backup/tests/`) ; `decouper` étendu aux
  blocs imbriqués (assertion (2)) ; `code()` rend la copie masquée tronquée au premier `#[cfg(test)]` du masque ;
  `argument_de` apparie sur le masque et rend la tranche de la source ; assertion (6) des fins de ligne (F9 (a)) ;
  **M-a7a** (le bras `"` retiré de `decouper`) et **M-a7b** (retrait non gourmand dans `code()`), tuées par (3') — la même
  chaîne, fermée par `/* c */` après l'appel — et, pour M-a7a, par (3) et (4).
- **Écartées** : écrire le balayeur dans `mod tests` (copie de plus) ; ne garder qu'une forme de M-a7 ; extraire
  `lexique` dans un crate de test commun (le périmètre de #727).
- *Annotation de la validation P3 (2026-10-24)* : ce choix est **retiré**. La déclaration « en tête d'`auto_backup.rs` »
  tronquait `code()` avant tout le code de production (R1 = F1 de la P3) — troisième défaut de l'AC-a11, chacun né de
  la remédiation précédente. Rien de ce qui est retenu ci-dessus n'est fait par la story : ni inclusion, ni extension
  de `decouper`, ni M-a7a/M-a7b. E-9 (3) est renvoyé à #727, où le piège est consigné (C-32-1-38).
- **Réversible** : oui.

### C-32-1-33 — L-B3 : la base, à la première sauvegarde, couverte par la marge seule (R3)

- **Contexte** : `seuil_d_espace` (`auto_backup.rs:496-500`) vaut `max(dernière, documents) × 1,2 + 256 Mio` ; sans
  réussite, `dernière = 0` (`:680-683`). L'archive contient base et documents : la base n'est couverte que par
  0,2 × documents + 256 Mio, quelle que soit sa taille. La phrase prescrite en P1 (« une base plus grosse que le dossier
  des documents n'est couverte que par la marge de 256 Mio ») était fausse dans les deux sens.
- **Retenu** : « tant qu'aucune sauvegarde n'a réussi, la base n'y est couverte que par la marge — un cinquième de la
  taille des documents, plus 256 Mio » ; contrôle aplati sur « un cinquième de la taille des documents ».
- **Écartée** : changer le seuil (une conception, hors des LOW mécaniques).
- **Réversible** : oui.

### C-32-1-34 — E-7 : un rejet qui n'est pas une `ApiError` est une issue inconnue ; `loadError` remis à `null` (R5 = F10, R6)

- **Contexte** : `request()` rend `undefined` sur un 200 au corps illisible (`api-client.ts:480-484`), et
  `applyState(undefined)` lève : le prédicat d'`issueInconnue` (faux hors `ApiError`) ne le classait nulle part, alors
  que la sauvegarde est écrite. Et `loadError`, posé par une relecture en échec, n'était effacé que par `loadSchedule`.
- **Retenu** : `!isApiError(err) || <prédicat>(err)` → issue inconnue (message neuf, relecture : un `GET`, sans
  risque) ; le prédicat partagé reste inchangé pour l'import. `runNow` remet `loadError` à `null` au départ ; Vitest,
  mutation M-b10.
- **Écartée** : traiter le rejet hors `ApiError` comme un 4xx (sans relecture) : il naît surtout d'un 200, dont l'archive
  est écrite.
- **Réversible** : oui.

### C-32-1-35 — Bilan d'issues : #727 comptée ; son inventaire complété en commentaire (F4, R4)

- **Contexte** : #727 (`technical-debt`, `P3`) a été ouverte par l'orchestrateur une minute après la remédiation P1, sur
  sa demande ; la fiche gardait « −3 + 2 ». Son texte ne couvre que `kesh-api` : quatre sites de `kesh-db` manquent
  (`src/backup.rs:1950-1951`, `src/repositories/closing_checks.rs:922`, `:928`, `tests/test_schema_guard.rs:818-827`),
  et elle ne nomme pas `lexique`, qui existe.
- **Retenu** : bilan **−3 + 3 = 0** issue, **−1** bug ; inventaire complété dans la fiche ; texte d'un commentaire à
  verser sur #727 par l'orchestrateur (fiche, *Commentaires à verser*). L'entrée « Validation P1 » du Change Log, juste à
  sa date, n'est pas réécrite.
- **Écartée** : une issue neuve pour les sites de `kesh-db` (même classe, même remède : #727 la porte).
- *Annotation de la validation P3 (2026-10-24)* : le texte du commentaire sur #727 est **remplacé** par celui de la P3
  (sa dernière phrase annonçait le réemploi de `lexique` par la 32-1a, retiré) ; le bilan d'issues ne change pas
  (C-32-1-38).
- **Réversible** : oui.

### C-32-1-36 — La 32-1b au-delà de cinq modules au grain métier : déclaré, non découpé (F5)

- **Contexte** : au grain des crates et paquets, la 32-1b touche 4 modules ; au grain des modules métier de premier
  niveau que cite `CLAUDE.md`, 7 (`features/admin-backup`, `features/admin-restore`, `shared/utils`,
  `kesh-api/routes/backup_schedule`, `kesh-api/auto_backup`, `kesh-i18n`, le manuel). La règle de splitting préventif vise
  une story avant sa validation ; la tendance P1 → P2 (4 → 6 MEDIUM, dont un recyclage sur l'AC-a11 de la 32-1a) lève
  aussi le signal de sévérité.
- **Retenu** : le compte écrit avec son grain ; le signal déclaré au Project Lead (Change Log) ; aucun découpage par la
  remédiation. Coupe proposée si elle est décidée : un écran (E-6, E-7, E-8, L-B6) et un serveur (L-B5, L-B7), chacun à
  cinq modules, au prix d'un ordre de fusion de plus sur le PDF et les catalogues.
- **Écartée** : découper d'office (la décision appartient au Project Lead ; les trois MEDIUM de la P2 qui portent sur
  la 32-1b — F1, F2, R3 — sont des défauts de la rédaction d'origine, non un recyclage).
- **Amendé (découpage, 2026-10-24)** : la 32-1b est **découpée** par décision de l'orchestrateur, avant la P3 —
  C-32-1-37. La coupe retenue diffère sur deux points de celle proposée ici : le bandeau (lecteur de
  `lastSuccessInFuture`) et tout le manuel restent à l'écran, si bien que le serveur ne touche ni le frontend ni le PDF
  (aucun ordre de plus sur le PDF) ; et les comptes sont 5 (écran) et 3 (serveur), non « cinq chacun ».
- **Réversible** : oui.

---

*C-32-1-37 : découpage de la 32-1b avant la validation P3 (Opus 5.5, 2026-10-24, sur décision de l'orchestrateur).*

### C-32-1-37 — La 32-1b découpée en écran (32-1b) et serveur (32-1d) : règle de splitting préventif

- **Contexte** : la 32-1b touchait **7 modules** au grain métier (C-32-1-36), au-delà des cinq de la règle de splitting
  préventif de `CLAUDE.md` ; et la validation P2 a levé le signal de sévérité **MEDIUM → MEDIUM** (4 → 6 MEDIUM
  distincts) avec un **recyclage** — R1, la M-a7 indécidable, née de la remédiation P1 de l'AC-a11, qui reçoit sa
  deuxième réécriture. L'amendement D5 exempte du découpage une sévérité égale faite de défauts **d'origine** distincts,
  non un recyclage ni une dispersion au-delà de cinq modules : les deux motifs de découpage sont réunis. La story n'est
  pas encore passée en P3 : la règle s'applique avant.
- **Signal déclaré au Project Lead** : oui — au Change Log de la P2 (C-32-1-36), puis ici et à l'entrée « Découpage de
  la 32-1b » du Change Log de la fiche. Décision prise par l'orchestrateur en autonomie (Guy absent), **à confirmer par
  Guy** à son retour ; réversible tant que rien n'est développé.
- **Retenu** — la coupe de la remédiation P2, vérifiée au code et ajustée :
  - **32-1b « écran »** (`32-1b-defauts-sauvegarde-automatique`) : E-6 (AC-b1), E-7 (AC-b2), E-8 (AC-b3), L-B5 côté
    bandeau (AC-b4 réduit : type `BackupSchedule`, fixtures, `BackupScheduleBanner.svelte`, clé
    `admin-backup-banner-future`), L-B6 (AC-b5), le manuel entier (AC-b7, L-B3 compris : une phrase sur `seuil_d_espace`,
    qui ne change pas), i18n (AC-b8, **+2**), captures (a)–(d), E2E, mutations M-b1–M-b4, M-b7–M-b10 et **M-b11** neuve
    (le bandeau qui ignore le drapeau — sans elle, la moitié bandeau de L-B5 n'avait aucune mutation propre, la M-b5
    d'origine tuant le seul serveur). Modules : `features/admin-backup`, `features/admin-restore`, `shared/utils`,
    `kesh-i18n`, manuel — **5** ; aucun fichier Rust.
  - **32-1d « serveur »** (`32-1d-defauts-sauvegarde-serveur`) : L-B5 drapeau servi (AC-d1 : `is_success_in_future`,
    `lastSuccessInFuture`, `prochaine_et_retard`), L-B7 (AC-d2 : `t(…)`, G9 à 5, `i18n_e2e.rs`), i18n (AC-d3, **+1**),
    capture du refus en allemand (AC-d4), gates et E2E (AC-d5), M-d1 = ex-M-b5, M-d2 = ex-M-b6 (AC-d6). Modules :
    `kesh-api/auto_backup`, `kesh-api/routes/backup_schedule`, `kesh-i18n` — **3** ; aucun fichier du frontend ni du
    manuel (un champ de plus dans la réponse est ignoré par le client actuel ; le refus n'a pas de phrase au manuel).
  - **Aucune AC entre les deux** : chaque élément de l'AC-b4 d'origine est attribué ; L-B3 n'a pas de code ; AC-b9 (e)
    est l'AC-d4.
  - **Numérotation** : les numéros de la 32-1b sont gardés (renvois à AC-b6, T-b1, M-b5, M-b6), pour que le registre et
    les rapports P1/P2, qui les citent, restent lisibles ; la 32-1d numérote en `AC-d*`, `M-d*`, `T-d*`.
  - **Ordre de fusion** : 32-1d → 32-1b (la 32-1b déclare requis un champ que la 32-1d sert ; son E2E final, sa capture
    (c), ses mutations et son gate complet se jouent sur `main` mis à jour) ; 32-1c → 32-1b (PDF, C-32-1-17) ; la 32-1a
    libre, la seconde des 32-1a et 32-1d se rebasant (trois fichiers de code communs, C-32-1-27).
  - **Mots-clés** : `closes #711` sur la dernière fusionnée des 32-1a, 32-1b, 32-1d — jamais la 32-1d, qui porte
    `refs #711` ; 32-1a `closes #716`, 32-1c `closes #714` inchangés.
  - **CHANGELOG** : bloc `## [0.16.0] — Non publié` créé par la première fusionnée des 32-1c et 32-1d ; une ligne
    `Corrigé` pour la 32-1d (refus traduit).
  - **Bilan d'issues** : inchangé (0 issue, −1 bug) ; aucune issue ouverte ni fermée par le découpage.
- **Écartées** : la coupe à deux manuels (bandeau et phrases du serveur au manuel de la 32-1d) — un PDF binaire de plus à
  ordonner, pour aucun gain ; numéroter à neuf la 32-1b (AC-b1 à b9) — casserait les renvois du registre et des rapports ;
  ne pas découper (C-32-1-36) — les deux motifs de l'amendement D5 sont réunis.
- **Amendé (validation P3, 2026-10-24, F2 = R4)** : le motif écrit au *Contexte* et aux *Écartées* (« les deux motifs
  sont réunis ») est **faux** pour la 32-1b. Le recyclage (R1 de la P2, M-a7 indécidable) porte sur l'AC-a11, qui est à
  la **32-1a** ; les trois MEDIUM de la P2 sur la 32-1b (F1, F2, R3) étaient des défauts d'origine (C-32-1-36). **Seule
  la dispersion — 7 modules au grain métier, au-delà des cinq de la règle — justifie le découpage de la 32-1b**, et elle
  y suffit : la coupe ne change pas. Le recyclage, lui, n'était pas traité par cette coupe ; il l'est par le retrait de
  l'AC-a11 (C-32-1-38).
- **Réversible** : oui (refondre les parties B et D), tant qu'aucune des deux n'est développée.

---

*C-32-1-38 : remédiation de la validation P3 (Opus 5.5, 2026-10-24, retrait décidé par l'orchestrateur).*

### C-32-1-38 — L'AC-a11 retiré de la 32-1a, E-9 (3) renvoyé à #727 ; grain métier de la 32-1a déclaré ; LOW de la P3

- **Contexte** : la validation P3 (Sonnet, R 0/0/1/4, F 0/0/2/8 ; R1 = F1, F2 = R4) relève un troisième défaut sur
  l'AC-a11 : la déclaration `#[cfg(test)] #[path …] mod lexique;` « en tête » d'`auto_backup.rs` (C-32-1-32) précède
  tout le code de production, si bien que `code()`, tronqué au premier `#[cfg(test)]`, ne rend plus que la tête du
  fichier — `gardes_possedes` rougit sur `try_lock_owned()` et sa boucle des jetons interdits passe à vide sur ce
  fichier. Trois rédactions de l'AC (C-32-1-6, C-32-1-20, C-32-1-32), trois défauts (P1 R1, P2 R1, P3 R1), chaque
  remédiation introduisant le suivant : le **recyclage** que l'amendement D5 de la règle de splitting désigne. Le
  constat E-9 (3) est LOW, sur des gardes de test ; #727 (ouverte, `technical-debt`, P3) demande déjà l'extraction
  d'un balayeur commun qui corrigera toutes les copies d'un coup.
- **Retenu** (décision de l'orchestrateur ; mise en forme par la remédiation) :
  - **Retrait** : l'AC-a11, ses mutations M-a7a/M-a7b et le réemploi de `lexique` sortent de la 32-1a ; le numéro
    AC-a11 garde une ligne de renvoi, M-a7 n'est pas réemployé ; la 32-1a ne touche ni
    `crates/kesh-db/tests/lexique/mod.rs` ni le code des helpers. Mutations de la 32-1a : huit (M-a1a à M-a6).
  - **Angle mort écrit** (choix de la remédiation, contestable) : l'AC-a10 s'élargit aux doc-comments de `code()`,
    `argument_de`, `corps()` — leur limite telle que E-9 (3) l'établit, la conséquence muette, le renvoi à #727 —,
    commentaires seuls, sans test ni mutation ; la fiche le dit aussi (*Inventaire*).
  - **#711** : se ferme toujours par la dernière fusionnée des 32-1a et 32-1b ; chaque constat a un sort écrit — E-1,
    E-2 suivis par #725, E-9 (3) par #727 —, que le corps de la PR écrit en toutes lettres, le commentaire sur #727
    versé avant la fusion ; à défaut, `refs #711`. Bilan inchangé : **0** issue, **−1** bug.
  - **Commentaire sur #727** : un texte P3 qui remplace celui de la P2 (dont la dernière phrase annonçait le réemploi)
    — E-9 (3), le piège de la déclaration en tête, l'inventaire sur tout `crates/` (avec `admin_pat_denied_e2e.rs:124`,
    R3), `lexique` comme cible.
  - **Grain métier de la 32-1a, après retrait** (F2) : **6** (`routes/invoice_pdf_service`, `routes/audit_log`, la
    garde des textes, `auto_backup`, `repositories/company_invoice_settings`, `settings/fiscal-years`), 3 au grain des
    crates. Au-delà de cinq : **signal déclaré au Project Lead**, sans découpage — chaque AC touche un module et
    aucun ne lit le résultat d'un autre, et le seul recyclage en est sorti.
  - **LOW** : R2 (`i18n-keys.test.ts`, `sitesTotal` +2 par la 32-1b), R3 (inventaire), R5 (espace disque : la base
    comptée à travers la dernière sauvegarde seulement si celle-ci dépasse les documents), F3 (C-32-1-26 amendé), F4
    (`CHANGELOG.md` commun aux B, C, D, un seul bloc, conflit prévu), F5 (commande exacte de l'AC-c9), F6
    (`sprint-status.yaml`), F7 (`-p kesh-i18n` au gate ciblé de l'AC-d5), F8 (`./api-error`), F9 (G22 nommée), F10
    (200 illisible : tentative terminée, limite du message unique écrite).
- **Écartées** : une quatrième rédaction de l'AC-a11, placée avant `mod tests` comme le proposaient R1 et F1 — elle
  aurait été juste à la lecture, mais rien n'assurait qu'elle ne porte pas le défaut suivant, et elle reste
  l'extraction de #727 en miniature ; une clé propre 32-1e pour les helpers (F2) — une clé de plus pour un LOW de
  gardes de test que #727 corrige entier ; découper la 32-1a au motif de ses six modules — ses AC sont indépendants ;
  une variante de message pour le 200 illisible (F10) — une clé de plus pour un cas rare.
- **Réversible** : oui — l'AC-a11 peut revenir tant que la 32-1a n'est pas développée, et #727 le porte d'ici là.

### C-32-1-39 — 32-1c : le `literate` des listings ne prend pas, repli ASCII dans les deux scripts d'exemple

- **Contexte** (développement de la 32-1c, 2026-10-24) : l'AC-c7 prescrivait `literate={✓}{{+}}1 {✗}{{x}}1 {═}{{=}}1`
  au style `kesh`, avec un repli écrit si le compte de U+FFFD le démentait. Après `make fr`, le manuel d'administration
  rendait encore **9** U+FFFD — exactement les neuf signes des deux scripts d'exemple ; les replis d'icônes et la macro
  `\keshAttention` avaient porté les deux autres PDF à 0. Sous XeLaTeX, `listings` ne traite pas les caractères au-delà
  de U+00FF : le `literate` sur eux ne remplace rien.
- **Retenu** : le repli que la fiche prescrit — les neuf signes écrits en ASCII dans les deux scripts (`+`, `x`, `=`),
  hors `sec:repetition-cloture` ; le `literate`, inopérant, est **retiré** du `.sty` et remplacé par un commentaire qui
  dit pourquoi (un réglage qui ne fait rien et paraît fonctionner est le pire des deux). Après reconstruction :
  administration **0** U+FFFD.
- **Écartées** : garder le `literate` « pour le jour où » (code mort qui paraît actif) ; `extendedchars`/`inputencoding`
  (sans effet sur des caractères hors de l'octet sous XeTeX) ; changer la police des listings (hors périmètre, effet sur
  tous les listings).
- **Réversible** : oui — trois lignes de `.tex` et un commentaire du `.sty`.

### C-32-1-40 — 32-1c : trois `\needspace` de plus, pour qu'aucun listing de la sous-section ne soit coupé

- **Contexte** : l'AC-c6 exige, au contrôle du PDF, qu'« aucun listing de la sous-section » ne soit coupé. Le PDF de la
  base de la branche (`c03d3a5f`) en coupait **déjà deux** (contrôle 1, p. 54→55 ; contrôle 7, p. 57→58), sans lien
  avec la 32-1c ; l'en-tête de la sous-section dit aussi que le PDF ne doit pas couper ses listings.
- **Retenu** : `\needspace{12\baselineskip}` avant les listings des contrôles 1 et 7 ; le décalage coupant alors celui
  du contrôle 4, `\needspace{9\baselineskip}` avant lui. Aucune ligne d'aucun listing ne change (`git diff` des
  `lstlisting` de la sous-section vide, AC-c9) ; contrôle page par page au `pdftotext -layout` (haut et bas de chaque
  page de 53 à 67) : aucune coupure.
- **Écartées** : laisser les deux coupures préexistantes (l'AC ne serait pas tenue) ; un `\needspace` devant chacun des
  vingt-deux listings (bruit, et une page presque vide à chaque fois).
- **Réversible** : oui — trois lignes. *Limite* : un `\needspace` tient la mise en page **d'aujourd'hui** ; une
  phrase ajoutée plus haut peut déplacer une coupure — le contrôle se refait à chaque régénération.
