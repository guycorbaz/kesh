# 0.17.4 — registre des choix autonomes de la story `v0174-session` (clé `v0174s`)

Registre propre à la story (l'orchestrateur le verse par union). Chaque entrée : contexte, retenu, écartées, réversibilité.
Fiche : `v0174-session.md` (parente), `v0174-session-1.md` (serveur), `v0174-session-2.md` (navigateur). Écrit le
2026-11-05 par l'agent de fiche (Opus 5.5, autonomie), sur `origin/main` = `7bd4d5c0`.

## C-v0174s-1 — Découpage en deux sous-fiches sœurs

- **Contexte** : #890 (stockage des jetons, serveur), #896 (paramètre de test, serveur), #895 (avertissement, navigateur)
  et les LOW de #898/#899 (moitié serveur, moitié navigateur). Une fiche unique toucherait `kesh-db`, `kesh-api`,
  `kesh-i18n`, `frontend/src/lib/app/stores`, `frontend/src/lib/shared/components`, `frontend/src/routes` : six modules,
  au-dessus du seuil de cinq de la règle de splitting préventif.
- **Retenu** : `v0174-session-1` (serveur : #890, #896, LOW serveur ; `kesh-db` et `kesh-api`, deux modules) et
  `v0174-session-2` (navigateur : #895, LOW navigateur ; quatre modules), sur la même branche, la 1 avant la 2. Aucune
  dépendance de code entre elles : le format du cookie de renouvellement ne change pas (C-v0174s-2), le navigateur n'a
  rien à apprendre du hachage. La parente porte le contexte commun, le triage des LOW et l'inventaire des textes.
- **Écartées** : une fiche unique avec dérogation (précédent 0.17.3, C-v0172s-41) — ici l'indépendance des deux moitiés
  ôte toute raison de déroger ; trois fiches (#896 à part) — #896 est une édition mécanique de tests, sans conception.
- **Réversible** : oui (fusion des deux fiches), tant que le développement n'a pas commencé.

## C-v0174s-2 — L'empreinte du jeton, sans migration : SHA-256 tronqué, mis en forme dans la colonne existante

- **Contexte** : `refresh_tokens.token` est `CHAR(36)` avec `chk_refresh_tokens_token_format`
  (`^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$`, `20260405000001_auth_refresh_tokens.sql:7,14-15`) ;
  une empreinte SHA-256 hexadécimale (64 caractères) n'y entre pas. Toute migration empêcherait la 0.17.3 de redémarrer
  (retour par la sauvegarde seulement) ; changer la colonne (`MODIFY`) est breaking au sens P3, imposerait le bump de
  `kesh_version_min_required`, et rendrait **inimportables** les `.keshbackup` 0.17.3 dont les lignes portent un UUID
  (la contrainte neuve les refuserait à l'insertion). Kesh 0.17.3 tient des livres réels.
- **Retenu** : `token_digest(clair) = ` les **16 premiers octets** de `SHA-256(clair)`, en hexadécimal minuscule, dont
  le **13ᵉ chiffre** (celui de la version d'un UUID) est **forcé à `0`**, mis en forme `8-4-4-4-12`. La valeur satisfait
  la contrainte existante ; **aucune migration** (89 migrations, inchangé). Le `0` (version inattribuée par la RFC 9562)
  distingue toute empreinte d'un UUID v4 (`4` à cette place, `uuid::Uuid::new_v4`, `routes/auth.rs:307`, `:506`, `:615`,
  `setup.rs:292`) : une ligne d'avant la mise à jour se reconnaît à `SUBSTRING(token, 15, 1) <> '0'`. **Force** : 124
  bits d'empreinte pour un jeton de 122 bits d'aléa (UUID v4) — l'empreinte ne réduit pas l'espace à parcourir ; une
  préimage coûte de l'ordre de 2¹²² essais. Vecteur de contrôle (calculé à la fiche par
  `printf '%s' '00000000-0000-4000-8000-000000000000' | sha256sum`, à recalculer au développement) : clair
  `00000000-0000-4000-8000-000000000000` ⇒ empreinte `db8055e0-e030-0d5a-016b-ec4dc338d698` (le chiffre brut était `7`).
- **Écartées** : (a) une colonne `token_hash CHAR(64)` neuve, l'ancienne retirée — migration breaking, sauvegardes
  0.17.3 à convertir à l'import, retour à la 0.17.3 par la sauvegarde seulement ; (b) une colonne neuve **à côté** de
  `token` (gardée nullable) — migration non breaking mais retour par la sauvegarde seulement, et deux colonnes à tenir
  cohérentes ; (c) l'empreinte sans le `0` forcé — les lignes d'avant ne se reconnaîtraient plus, ni ne s'effaceraient.
- **Coût assumé** : le nom de colonne `token` et son commentaire `'UUID v4 opaque'` mentent désormais ; ils ne se
  corrigent que par une migration. La documentation du dépôt (`refresh_tokens.rs`, `entities/refresh_token.rs`, README)
  le dit ; **issue demandée** pour renommer la colonne à la prochaine migration de la table (« on trace toujours »).
- **Réversible** : oui — la 0.17.3 redémarre sur la base (aucune migration), après vidage de `refresh_tokens`
  (C-v0174s-7) ; ses sessions repartent par une
  reconnexion (C-v0174s-7).

## C-v0174s-3 — Le hachage vit dans le dépôt `refresh_tokens`, les appelants passent le jeton clair

- **Contexte** : six fonctions du dépôt reçoivent la valeur du jeton (`create`, `find_active_by_token`,
  `find_by_token_include_revoked`, `revoke_by_token`, `set_idle_deadline`, `was_rotated_within`) ; leurs appelants de
  production sont `routes/auth.rs` (huit appels) et `routes/setup.rs` (un) — neuf ; `find_active_by_token` n'en a aucun
  (tests seulement). *(« Sept fonctions … deux » à `setup.rs` corrigé à la validation P1, R7.)* Le précédent
  `password_reset_tokens` hache dans `kesh-api` et passe l'empreinte au dépôt (`password_reset_tokens.rs:1-10`).
- **Retenu** : le dépôt hache lui-même (`pub fn token_digest(&str) -> String`, `kesh-db/src/repositories/refresh_tokens.rs`),
  à l'entrée de chaque fonction ; les signatures et les neuf appels de `kesh-api` **ne changent pas**. Un appelant ne peut
  donc pas oublier de hacher. `sha2 = "0.10"` entre dans `kesh-db` (déjà au graphe par `kesh-api` : aucune crate neuve au
  `Cargo.lock`). Le champ lu de l'entité est renommé `token_digest` (`#[sqlx(rename = "token")]`) pour qu'aucun lecteur ne
  le prenne pour le jeton.
- **Écartées** : le patron `password_reset_tokens` (empreinte calculée par l'appelant) — neuf sites à tenir, et un oubli
  stockerait en clair sans que rien ne rougisse (la contrainte accepte un UUID) ; un type `TokenDigest` distinct — plus
  de code pour une garantie que l'encapsulation donne déjà.
- **Réversible** : oui.

## C-v0174s-4 — Comparaison par égalité SQL sur l'index unique, sans comparaison à temps constant

- **Contexte** : le mandat demande de concevoir « la comparaison ».
- **Retenu** : `WHERE token = ?` lié à `token_digest(clair)`, sur `uq_refresh_tokens_token`. Une mesure du temps de
  recherche ne renseigne que sur l'empreinte du jeton **proposé**, que l'attaquant connaît déjà ; elle ne révèle rien de
  l'empreinte stockée qu'un attaquant pourrait inverser. Comparer à temps constant supposerait de lire la ligne sans
  index (balayage), sans gain.
- **Écartée** : `subtle::ConstantTimeEq` sur l'empreinte relue — même conclusion, une dépendance de plus.
- **Réversible** : oui.

## C-v0174s-5 — SHA-256, sans argon2, sans sel, sans poivre

- **Contexte** : mandat : « SHA-256 d'un jeton à haute entropie suffit — pas d'argon2 sur chaque renouvellement ;
  justifier ».
- **Retenu** : SHA-256. Un hachage lent protège un secret **pauvre** (mot de passe) contre la recherche exhaustive ; un
  UUID v4 porte 122 bits d'aléa, hors de portée de toute recherche, lente ou rapide. Le dépôt hache à **chaque** battement
  d'activité (au plus toutes les 15 s par navigateur), chaque renouvellement et chaque déconnexion : argon2 (dizaines de
  millisecondes, mémoire) y serait un coût sans contrepartie, et son **sel** interdirait la recherche par index. Un poivre
  (HMAC par un secret du serveur) n'ajoute rien à 122 bits et lierait les sessions à la rotation d'un secret.
- **Écartées** : argon2id, bcrypt, HMAC-SHA-256 au secret JWT.
- **Réversible** : oui (changer `token_digest` déconnecte tout le monde une fois, rien de plus).

## C-v0174s-6 — Les jetons d'avant la mise à jour : invalidés, puis effacés au démarrage et à l'import

- **Contexte** : mandat : « le sort des jetons existants à la mise à jour : les invalider (une reconnexion) est
  acceptable et probablement le plus sûr ».
- **Retenu** : (1) **invalidés par construction** — un cookie d'avant porte un UUID clair dont l'empreinte ne correspond à
  aucune ligne : renouvellement, battement et déconnexion le traitent en jeton inconnu (401 / sans effet) ; chacun se
  reconnecte **une fois**. (2) **Effacés** : le nettoyage du démarrage (`delete_expired_and_revoked`, `main.rs:232`)
  efface aussi toute ligne qui ne porte pas une empreinte (`SUBSTRING(token, 15, 1) <> '0'`), si bien qu'aucun jeton
  clair ne reste en base — ni dans la sauvegarde suivante — après le premier démarrage ; la restauration complète fait
  de même dans sa transaction (`delete_clear_tokens_in_tx`), si bien qu'une sauvegarde 0.17.3 importée n'y replante
  aucun jeton clair. Le prédicat est une constante partagée (DRY). Placer l'effacement dans le nettoyage existant le fait
  exercer par un test de dépôt, sans autre changement de `main.rs` que son message. Le nettoyage du démarrage étant non
  fatal (`main.rs:240-242`), « aucun jeton clair après le premier démarrage » suppose qu'il a réussi — sinon un
  avertissement au journal, et l'effacement est repris au démarrage suivant (validation P2, P2-11).
- **Écartées** : révoquer au lieu d'effacer (la valeur claire resterait en base, révoquée) ; ne rien effacer (les lignes
  sont inertes, mais la sauvegarde emporterait des jetons clairs jusqu'à leur expiration, jusqu'à 24 h) ; une migration de
  données (`DELETE` dans un `.sql`) — elle ferait une 90ᵉ migration, à trier P7, pour ce que le démarrage fait sans elle.
- **Réversible** : oui.

## C-v0174s-7 — Retour à la 0.17.3 : possible sans la sauvegarde

- **Contexte** : P1 du `CLAUDE.md` (toute migration lie le retour à la sauvegarde d'avant).
- **Retenu** (**récrit à la validation P2, P2-6**) : la 0.17.4 n'ajoute aucune migration : la 0.17.3 redémarre sur une
  base passée par la 0.17.4 — **après qu'on a vidé `refresh_tokens`**, Kesh arrêté (`DELETE FROM refresh_tokens`, par le
  compte de Kesh, commande donnée au manuel d'administration, « Passer à la 0.17.4 ») ; tout le monde est déconnecté.
  *Pourquoi* : la 0.17.3 cherche la valeur reçue telle quelle (`refresh_tokens.rs:19-20` ; cookie **ou corps**,
  `routes/auth.rs:424-427`), et une empreinte a la forme d'un UUID : présentée à la 0.17.3, elle trouve sa ligne et ouvre
  la session tant que son échéance n'est pas passée — quiconque aurait lu une base, un dump ou une sauvegarde pris sous
  la 0.17.4 reprendrait une session. La 0.17.4 ne peut pas changer la 0.17.3 : le remède est un geste de l'administrateur,
  écrit au manuel et au CHANGELOG, et éprouvé à la recette (b). La sauvegarde avant mise à jour reste recommandée.
- **Écartées** : révoquer les lignes au lieu de les effacer (même effet, requête plus longue) ; « attendre l'échéance
  réglée » (jusqu'à 24 h, invérifiable par l'administrateur) ; un mécanisme dans la 0.17.4 (inopérant : c'est la 0.17.3 qui
  lit).
- **Réversible** : sans objet.

## C-v0174s-8 — #896 : retrait mécanique, contrôlé par une lecture du diff

- **Contexte** : `Config::from_fields_for_test` porte le paramètre ignoré `_refresh_inactivity_ignored: TimeDelta`
  (`config.rs:455`), septième argument, passé par **85** appels dans **80** fichiers — 86 occurrences de
  `from_fields_for_test(` moins la définition (`git grep -c "from_fields_for_test(" -- crates`, sommé ;
  `git grep -l … | wc -l`), toutes à un argument par ligne (aucune occurrence n'a d'argument sur la ligne de l'appel :
  `git grep -n "from_fields_for_test(" -- crates | grep -v "from_fields_for_test($"` rend une sortie **vide** — la
  définition elle-même finit par `(` ; « ne rend que la définition » corrigé à la validation P1, R7). Il est
  suivi d'un autre `TimeDelta`
  (`rate_limit_window`) : retirer la mauvaise ligne **compile** et change en silence la configuration d'un test.
- **Retenu** : retrait du paramètre et de la ligne correspondante à chaque appel, **par l'outil Edit** (consigne 14) ;
  contrôle par lecture seule (script de lecture permis, sans écriture) : pour chaque appel, la ligne retirée est le
  septième argument de la version d'avant, et les autres arguments sont inchangés ; décompte des lignes retirées égal à
  85 (plus la ligne de la définition et son commentaire). Aucun test ne change de sens.
- **Écartées** : `sed`/`perl` (interdits sur du code) ; garder le paramètre (dette tracée par #896, au périmètre accepté).
- **Réversible** : oui.

## C-v0174s-9 — Triage des LOW de #898 et #899

- **Retenu** : la table § *Triage* de la fiche parente — chaque constat **corrigé** (avec sa sous-fiche) ou **non
  corrigé, motif écrit**. Les constats non corrigés qui sont des défauts de comportement (N-B3/N-E7, N2-E1) font l'objet
  d'une **issue demandée** ; ceux qui ne sont pas des défauts (choix de conception, procédé de la recette 0.17.3, mutation
  équivalente) sont écrits comme tels et n'en ont pas.
- **Réversible** : oui.

## C-v0174s-10 — L'avertissement : une minute avant l'échéance (la moitié de la durée si elle est d'une minute)

- **Contexte** : #895 propose « une ou deux minutes avant l'échéance » ; la durée N va de 1 à 1440 minutes.
- **Retenu** : `W = min(60 s, N × 30 s)` — 60 secondes pour N ≥ 2, 30 secondes pour N = 1. Une minute laisse le temps de
  voir l'avis et de cliquer, sans interrompre trop tôt une lecture ; plafonner à la moitié de N évite qu'un réglage d'une
  minute ne fasse apparaître l'avis dès la dernière activité. Constante exportée `WARNING_LEAD_MS = 60_000`. **Aucun
  réglage neuf** (mandat : « pas de réglage nouveau sauf nécessité »).
- **Écartées** : deux minutes (interrompt davantage, pour un bénéfice faible) ; un réglage dans l'écran Session.
- **Réversible** : oui (une constante).

## C-v0174s-11 — Accessibilité : l'avis ne déplace pas le focus ; une annonce statique, un compte à rebours muet

- **Contexte** : un `alertdialog` déplace le focus sur son bouton. Mais si le focus est dans l'`<iframe>` de l'aperçu
  PDF — le cas même de #895 —, le ramener dans la page fait recevoir à la fenêtre un `focus` dont la cible est la
  fenêtre, que le module compte comme activité (`session-idle.svelte.ts:245-247`) : l'avis se renouvellerait lui-même,
  et la session d'un poste quitté ne se fermerait jamais (FR13).
- **Retenu** : aucun déplacement programmatique du focus. L'avis porte une phrase **statique** dans un élément
  `role="alert"` (annoncée une fois à l'apparition : « Votre session va bientôt se fermer faute d'activité. ») ; le compte
  à rebours visible (`m:ss`) est `aria-hidden="true"`, pour qu'un lecteur d'écran ne l'annonce pas à chaque seconde ; le
  bouton « Rester connecté » est un `<button>`. **Complété à la validation P1 (F12)** : la phrase annoncée dit comment
  agir — « … faute d'activité : cliquez sur « Rester connecté », ou n'importe où dans Kesh hors du document affiché. »
  (**sans promettre la touche**, validation P2, P2-3 : quand le focus est dans l'aperçu PDF, ses touches ne parviennent pas
  à la page) ; le bouton est atteignable au clavier **sauf** quand un dialogue modal est ouvert (son piège à focus enferme
  la tabulation) — écart déclaré, tracé par **#912** avec les autres limites de l'avis (F13). Contraste du thème clair.
- **Écartées** : `role="alertdialog"` avec focus ; compte à rebours dans une région `aria-live`.
- **Réversible** : oui.

## C-v0174s-12 — « Rester connecté » est une activité, rien de plus

- **Retenu** : le bouton appelle `sessionIdle.stayActive()`, qui note l'activité **sans** la régulation d'une seconde
  (même contrôle préalable de l'échéance que `recordActivity`) ; le battement part au tic suivant par le chemin existant.
  Toute autre activité dans la page fait aussi disparaître l'avis. Un appel explicite est nécessaire en plus des
  écouteurs : l'activation par une technologie d'assistance n'émet qu'un `click`, ni `pointerdown` ni `keydown`.
- **Écartées** : un battement envoyé directement par le bouton (un second chemin d'envoi, hors de la régulation entre
  onglets).
- **Réversible** : oui.

## C-v0174s-13 — L'avis passe au-dessus d'un dialogue ouvert, sans le fermer

- **Contexte** : les dialogues de Kesh (bits-ui `Dialog`, `components/ui/dialog/`) sont modaux et se ferment, par
  défaut, sur un geste hors de leur contenu (`interactOutsideBehavior`, employé en `'close'` par
  `WriteOffDialog.svelte:127`, `DeleteBankImportAction.svelte:132`). Un clic sur « Rester connecté » pendant une saisie dans
  un dialogue le fermerait et perdrait la saisie que l'avis veut sauver.
- **Retenu** (**récrit à la validation P1, F3**, établi dans `frontend/node_modules/bits-ui` 2.16.5) : un élément
  ordinaire, **pas de primitive bits-ui** (`Dialog.Content` est toujours `aria-modal="true"`,
  `bits/dialog/dialog.svelte.js:265-266`, contre C-v0174s-11), qui tient trois propriétés : (1) un écouteur **natif**
  `pointerdown` en bouillonnement sur sa racine appelle `stopPropagation()` — la couche du dialogue ne décide de se fermer
  qu'au bouillonnement sur `document`, après avoir marqué l'événement « intercepté » en capture
  (`bits/utilities/dismissible-layer/use-dismissable-layer.svelte.js:84`, `:90`, `:110`) ; natif plutôt que `onpointerdown`,
  que Svelte 5 délègue à la racine de l'application ; (2) `pointer-events: auto` — `ScrollLock`, monté par
  `Dialog.Content` quand `preventScroll` vaut `true` (défaut, `dialog-content.svelte:25`, `:85` — branche `{:else}`, celle
  de Kesh ; « `:78` » corrigé à la validation P2, P2-7), pose
  `document.body.style.pointerEvents = "none"` (`internal/body-scroll-lock.svelte.js:129`) ; (3) `z-index` > 50
  (`z-[60]`), la superposition et le contenu du dialogue étant `z-50` et montés après le layout. Échap reste au dialogue.
  À éprouver en Chromium à l'étape 0 ; si un point ne tient pas, l'écart est écrit (manuel) et l'issue demandée.
- **Écartée** (piste de la fiche d'origine) : une primitive bits-ui « non modale » empilée au-dessus — elle n'existe pas
  pour `Dialog` (toujours modale pour l'accessibilité), et `escapeKeydownBehavior="ignore"` sur la couche du dessus
  aurait privé le dialogue d'Échap.
- **Réversible** : oui.

## C-v0174s-14 — Sans `localStorage`, pas d'avertissement

- **Contexte** : sans stockage partagé, le module ne ferme rien de lui-même (C-v0172s-45) ; c'est le serveur qui ferme.
- **Retenu** : pas d'avis non plus (il annoncerait une fermeture locale qui n'aura pas lieu). Écrit au manuel
  d'administration (paragraphe existant sur `localStorage`).
- **Réversible** : oui.

## C-v0174s-15 — Compte à rebours en `m:ss`

- **Retenu** : « 0:59 » plutôt que « 59 secondes » : aucun pluriel à traiter dans quatre langues (le précédent
  `login-session-idle-one` montre le coût d'un singulier), et lecture immédiate.
- **Réversible** : oui.

## C-v0174s-16 — Registre : clé parente `split`, deux clés filles `ready-for-dev`

- **Contexte** : le mandat dit « `sprint-status.yaml` : `v0174-session: ready-for-dev` » ; le dépôt marque `split` une
  fiche découpée (précédents `v017-3-rapprochement-attente`, `v017-4-facture-fournisseur-brouillon`).
- **Retenu** : `v0174-session: split`, `v0174-session-1: ready-for-dev`, `v0174-session-2: ready-for-dev`. Signalé au
  rapport.
- **Réversible** : oui.

## C-v0174s-17 — Une sous-section « Passer à la 0.17.4 », écrite par parts, rédigée en commun à l'intégration

- **Contexte** (validation P1, F2 et R9) : chaque version a sa `\subsubsection{Passer à la 0.17.x}` au manuel
  d'administration (`admin-manual.tex:2513` pour la 0.17.3, `:2527` pour la 0.17.2) ; la fiche ne confiait à personne celle
  de la 0.17.4, et plaçait la consigne de mise à jour dans § *Durée d'inactivité*, où personne ne la cherche. Trois stories
  écrivent la 0.17.4.
- **Retenu** : la `v0174-session-1` crée `sec:maj-0-17-4` (avant `sec:maj-0-17-3`, patron de la 0.17.2) et y écrit sa part
  (aucune migration, retour à la 0.17.3 sans restauration, reconnexion unique à la prochaine activité, dumps et sauvegardes
  antérieurs porteurs de jetons clairs) ; la `v0174-session-2` y ajoute la sienne (l'avertissement) ; **la rédaction
  commune aux trois stories revient à l'intégration** (union). Le paragraphe *Mise à jour* de § *Durée d'inactivité* y
  renvoie.
- **Écartée** : laisser la sous-section à `prepare-release` (elle n'écrit pas de texte) ; une phrase dans § *Durée
  d'inactivité* seulement.
- **Réversible** : oui.

## C-v0174s-18 — Chaque sous-fiche régénère les PDF qu'elle touche

- **Contexte** (validation P1, R9) : la 1 laissait le PDF à la 2, si bien que son AC des textes n'était pas vérifiable
  seule et que la branche portait un PDF périmé entre les deux.
- **Retenu** : la 1 régénère le PDF d'administration, la 2 les deux (`make fr`, sous les deux verrous) ; chacune contrôle
  ses motifs dans le PDF aplati. Coût : un `make fr` de plus. C-v0174s-1 (« aucune dépendance ») tient de nouveau.
- **Réversible** : oui.

## C-v0174s-19 — Les tests prouvent qu'ils atteignent leur chemin : `rows_affected`, un assistant d'empreinte unique

- **Contexte** (validation P1, R1 et R2) : après le hachage, un `UPDATE … WHERE token = ?` de test lié à la valeur claire
  touche 0 ligne en silence ; deux tests de refus rendent le même 401 pour un jeton « inconnu » que pour leur cas ;
  `cleanup_removes_old_tokens` deviendrait muet sur les clauses d'âge ; trois fichiers ont besoin d'une empreinte calculée
  sans `token_digest`.
- **Retenu** : chaque écriture de test adaptée asserte `rows_affected()` ; `cleanup_removes_old_tokens` en empreintes,
  avec deux lignes à garder ; mutations M13 à M17 ; **un** assistant `independent_refresh_digest` dans
  `crates/kesh-api/tests/common/mod.rs` (DRY), qui recalcule par `sha2` et rend le vecteur de l'AC-1.
- **Réversible** : oui.

## C-v0174s-20 — L'avis est un composant à propriétés ; le layout le rend sur `typeof … === 'number'`

- **Contexte** (validation P1, F7) : un composant qui lit le module ne se teste que par un getter simulé non réactif ; et
  `!== null` rend l'avis quand un simulacre du module n'expose pas le champ (`undefined !== null`, simulacre de
  `root-layout-session.test.ts:11-16`).
- **Retenu** : `SessionIdleWarning.svelte` reçoit `remainingMs` et `onStay` ; le layout le rend si
  `typeof sessionIdle.warningRemainingMs === 'number'` ; le simulacre du test du layout gagne `warningRemainingMs` et
  `stayActive`.
- **Réversible** : oui.
