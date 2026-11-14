# Epic 36 — registre des choix autonomes de la story 36-5 (`36-5-restauration-compte-rendu`, #931)

Choix faits à la rédaction de la fiche (2026-11-14, Opus 5.5, worktree `kesh-36-5`, code lu sur `4de0bcc7e`), dans le cadre
de C-e360-8 (« forme exacte à la fiche »). Chaque choix : contexte, option retenue, options écartées, réversibilité.

### C-36-5-1 — Un maintien de session porté par `authState`, lu par le chemin 401 du client

- **Contexte** : l'inventaire par le symptôme (fiche, § *Inventaire*) trouve deux renvois à la connexion atteignables après
  l'import sans geste de l'utilisateur : le chemin 401 du client (`api-client.ts:202`, nourri par le battement et, après
  l'`exp` du jeton d'accès, par le bandeau des sauvegardes) et la fermeture d'inactivité (`session-idle.svelte.ts:407`).
- **Retenu** *(remplacé par l'amendement ci-dessous)* : `authState.holdForRestore()` / `releaseRestoreHold()` / `restoreHold` ; sous maintien, un 401 jette
  `UNAUTHENTICATED` sans renouvellement, effacement ni redirection (D1 de la fiche).
- **Écartées** : ignorer tout 401 du battement (C-e360-8 (b)) ; `sessionIdle.stop()` seul au clic (laisse le bandeau et un
  battement déjà parti) ; compte rendu reporté après la reconnexion ; correction de la spec seule.
- **Réversible** : oui ; état de module, aucune donnée écrite.
- **Amendé le 2026-11-14 (validation P1, V1-1)** : sous maintien, le renouvellement est **toujours tenté** ; seuls
  `clearSession` et la redirection sont suspendus quand il échoue. Sans cela, un import envoyé avec un jeton d'accès expiré
  échouait, et le battement cessait de prolonger la session.

### C-36-5-2 — Le battement continue pendant l'import ; la fermeture locale est différée, non annulée

- **Contexte** : suspendre le battement pendant l'import laisserait une restauration longue qui échoue retrouver une session
  close au serveur (aucune activité rapportée pendant l'attente).
- **Retenu** *(remplacé en partie par les amendements ci-dessous : issue inconnue, borne, avertissement)* : pendant le maintien, le module note l'activité et bat (le 401 éventuel est absorbé par C-36-5-1), mais ne ferme
  pas et n'avertit pas ; à la levée, le tic suivant ferme si l'échéance est passée. Au succès, le panneau arrête le module
  (`sessionIdle.stop()`) ; à l'issue inconnue, maintien gardé, module laissé tel quel ; à l'échec certain et au démontage du
  panneau, maintien levé.
- **Écartées** : suspendre aussi le battement ; lever le maintien dès la réponse reçue.
- **Réversible** : oui.
- **Amendé le 2026-11-14 (validation P1, V1-3 ; option prudente demandée par l'orchestrateur)** *(la borne de l'import en
  cours est remplacée par C-36-5-8 ; la levée à l'issue inconnue est remplacée par C-36-5-9)* : le maintien est **borné**
  — il ne couvre que l'import en cours et la lecture du compte rendu, chacune au plus B = la durée d'inactivité réglée
  (60 min si inconnue) ; il est **levé à l'issue inconnue** (la fermeture ordinaire reprend) ; à B après un succès, la
  session se termine comme par le chemin 401 (`clearSession`, `/login?reason=session_expired`). Levée en `finally`
  (V1-9). Écartées : maintien sans borne après une issue inconnue ; borne fixe de 15 min sans rapport avec le réglage.
- **Amendé le 2026-11-14 (validation P2, V2-5, V2-6 ; conduites prudentes)** : (a) l'import lui-même refusé en 401 sous
  maintien lève le maintien et termine la session (`expireSession`), comme aujourd'hui ; (b) *(remplacé par C-36-5-9)*
  sous maintien, l'avertissement se calcule sur `max(last + w, échéance du maintien)` et paraît avant l'expiration du
  maintien, au lieu d'être éteint ; pas d'avertissement pendant la lecture du compte rendu (session déjà morte, rien à prolonger).

### C-36-5-3 — Un E2E déterministe par réponse retenue, en plus de la répétition

- **Contexte** : C-e360-8 exige une preuve par répétition ; la fréquence de la course est inconnue pour `:140` (trois sur
  trois verts isolés, deux relevés) et d'environ un tiers pour `:262`.
- **Retenu** : un test neuf, en fin de `backup-restore-new-install.spec.ts`, retient la réponse de l'import après que le
  serveur a invalidé la session, force un battement (`kesh:session:heartbeatAt` retiré, mouvements), puis vérifie qu'aucun
  battement ne part plus après le compte rendu ; rouge certain avant le correctif.
- **Écartées** : `page.clock` (fausse `Date.now` dans tout l'écran) ; le même test sur l'assistant (le même composant, au prix
  de `/setup` à chaque passage).
- **Réversible** : oui.

### C-36-5-4 — N = 30

- **Contexte** : C-e360-8 fixe N ≥ 20, la valeur à la fiche.
- **Retenu** : 30 passages de `:140` et de `:262`, avant et après ; le test déterministe 10 fois après. Aucun rouge avant :
  dit tel quel, sans relance jusqu'au rouge ; la reproduction repose alors sur le test déterministe.
- **Écartée** : 20 (fréquence de `:140` inconnue).
- **Réversible** : oui.

### C-36-5-5 — Une phrase au manuel d'administration ; la note de la KF-056 dans `docs/testing.md`

*(Titre d'origine : « Manuel inchangé ; la note de la KF-056 dans `docs/testing.md` », remplacé à la validation P4, V4-11.)*

- **Contexte** : `admin-manual.tex:2340` décrit déjà le comportement rétabli (« Prenez le temps de lire, puis cliquez « Se
  reconnecter » ») ; `docs/testing.md` ne nomme pas la KF-056, tolérée implicitement dans « 0 à 2 de pollution ».
- **Retenu** *(remplacé par l'amendement ci-dessous)* : aucun changement du manuel ni de son PDF ; une note historique « KF-056 corrigée » après celle de la KF-052.
- **Écartée** : une phrase de plus au manuel (rien de visible n'y change : le code rejoint le texte).
- **Réversible** : oui.
- **Amendé le 2026-11-14 (validations P1, V1-7, et P2, V2-7)** : la note de la KF-056 se place avant `docs/testing.md:431`,
  après les deux paragraphes qu'annonce la note de la KF-052 ; le manuel d'administration reçoit une demi-phrase à `:2340`
  (renvoi d'un écran laissé ouvert au bout de la durée d'inactivité réglée, visible depuis la borne de C-36-5-2) ; `.tex`
  dans la story, PDF à l'intégration.
- **Amendé le 2026-11-14 (validation P4, V4-9)** : la phrase dit aussi que **quitter** l'écran du compte rendu mène à la
  connexion : « Laissé ouvert, ou quitté, l'écran mène à la connexion --- au bout de la durée d'inactivité réglée
  (\S\ref{sec:duree-inactivite}) dans le premier cas, aussitôt dans le second. »

### C-36-5-6 — Noms de bases

- **Contexte** : la clé contient des tirets, que les noms de bases MariaDB n'admettent pas sans guillemets.
- **Retenu** : `kesh_e2e_e365` et `kesh_ci_e365` (patron `e357` de l'Epic 35), journaux sous `kesh-gate-logs/e36-5/`.
- **Réversible** : oui.

### C-36-5-7 — Le renouvellement ne lève jamais le maintien : `login()` ne le touche plus

- **Contexte** : validation P2, V2-1 (HIGH). Le renouvellement réussi appelle `authState.login({…}, { broadcast: false })`
  (`api-client.ts:148-158`) ; si `login()` levait le maintien, un renouvellement réussi pendant l'import l'effacerait et
  rouvrirait la KF-056.
- **Retenu** : option (a) — `login()` ne touche plus le maintien ; seuls `releaseRestoreHold()` (le panneau), `logout()` et
  `clearSession()` le lèvent, et il expire seul à B. Aucune connexion d'un autre utilisateur n'en hérite : état de module
  propre à l'onglet ; les deux connexions réelles (`login/+page.svelte:66`, `setup.api.ts:73`) se font sur des pages où le
  panneau n'est pas monté, atteintes par un rechargement complet (module neuf) ou par une navigation qui démonte le
  panneau (levée) ; `logout()` et `clearSession()` lèvent ; la fermeture du navigateur perd l'état.
- **Écartée** : option (b) — `login()` lève le maintien sauf sur une option propre au renouvellement : un appelant futur qui
  oublierait l'option rouvrirait le défaut en silence ; (a) n'a pas de cas oubliable.
- **Réversible** : oui.
- **Précisé le 2026-11-14 (validation P3, V3-7, décision de l'orchestrateur)** : dans `logout()`, la levée est la première
  instruction, **avant** le `await fetch` (`auth.svelte.ts:204-212`, requête sans délai d'abandon), pour qu'une
  déconnexion pendue ou en échec ne laisse pas le maintien posé.

### C-36-5-8 — La phase 1 bornée par `max(B, LONG_OPERATION_TIMEOUT_MS)`, la phase 2 par B

- **Contexte** : validation P3, V3-1 (HIGH). Bornée par B (1 à 1 440 min, `session-idle.svelte.ts:76-77`), la phase 1
  expirait pendant un import plus long que B, alors que le client accorde jusqu'à une heure à l'import
  (`api-client.ts:53`, `LONG_OPERATION_TIMEOUT_MS = 60 * 60_000` ; `admin-restore.api.ts:46-48`) : la KF-056 revenait
  pour un Administrateur qui a réglé une durée courte.
- **Retenu** (décision de l'orchestrateur) *(remplacé en partie : la borne par l'amendement P4 ci-dessous, la levée à
  l'échec et à l'issue inconnue par C-36-5-9)* : confirmation → `holdForRestore(Math.max(B, LONG_OPERATION_TIMEOUT_MS))`, la
  constante importée d'`api-client.ts`, jamais recopiée ; succès → `holdForRestore(B)`, qui **remplace** l'échéance ;
  échec, issue inconnue, 401 de l'import, démontage → levée. L'échéance d'une heure ne survit à aucune fin d'import.
- **Écartées** : la borne B seule (première rédaction) ; écrire la limite au manuel (« pour N < 60 min, un import plus long
  que N perd son compte rendu ») et l'ouvrir en issue.
- **Réversible** : oui.
- **Amendé le 2026-11-14 (validation P4, V4-4)** *(le Retenu ci-dessus est remplacé pour la borne)* : un 401 renouvelé
  fait un rejeu, avec un délai neuf (`api-client.ts:319`, `:476-485`) ; la phase 1 est donc bornée par
  `max(B, 2 × LONG_OPERATION_TIMEOUT_MS + RESTORE_HOLD_MARGIN_MS)`, marge d'une minute (`RESTORE_HOLD_MARGIN_MS =
  60_000`, renouvellement et relecture de `/me`), constante écrite une fois. À l'échec certain et à l'issue inconnue, la
  levée se fait au premier geste (C-36-5-9).

### C-36-5-9 — D2 simplifié : le maintien ne diffère que le renvoi local, après l'échéance seulement

- **Contexte** : validation P4, V4-1, V4-2, V4-5. Le serveur ferme la session à `last + w` quoi qu'il arrive
  (`middleware/auth.rs:156-161` ; échéance écrite dans `refresh_tokens.expires_at`, `routes/auth.rs:25-33`) ; la formule
  `max(last + w, échéance du maintien)` de la P2 supprimait l'avertissement utile et en montrait un inopérant ; AC3 et
  AC6 se contredisaient sur `stayActive()` ; une levée immédiate à l'échec emportait le message si la session était échue.
- **Retenu** (décision de l'orchestrateur) : sous maintien, tant que `last + w − now > 0`, le module de session se
  comporte exactement comme aujourd'hui (battement, avertissement, *Rester connecté*) ; au-delà, ni fermeture, ni écriture
  d'activité, ni avertissement tant que le maintien dure ; la fermeture a lieu à la levée ou à l'échéance du maintien. À un
  échec certain ou une issue inconnue, la levée attend le premier geste (`pointerdown` ou `keydown` sur la fenêtre), le
  démontage ou l'échéance : le message reste lisible. L'accesseur `restoreHoldDeadline` de la P3 est retiré.
- **Écartées** : l'avertissement calculé sur `max(…)` (P2) ; l'avertissement éteint dès la pose (P1) ; la levée immédiate
  à l'échec (P1 à P3) ; un clic sur un bouton du panneau seulement (une règle par bouton).
- **Réversible** : oui.

### C-36-5-10 — La répétition « avant » sert le build d'un export du commit des rouges (développement)

- **Contexte** : le verrou de la voie 1 était tenu par les `gate-ci` de trois autres stories ; attendre son tour pour
  construire le frontend d'avant le correctif aurait interdit d'écrire le correctif pendant ce temps (le build lit les
  sources du worktree).
- **Retenu** : `git archive 29762b89a frontend` extrait sous `kesh-gate-logs/e36-5/avant-src/` (`node_modules` lié par
  `cp -al`), construit sous le verrou, servi par `KESH_STATIC_DIR` (variable `STATIC_DIR` du script `e2e-e36-5.sh`, qui
  l'écrit en tête du journal). `29762b89a` porte l'API du maintien **inerte** : le comportement servi est celui d'avant le
  correctif. Les specs exécutées sont celles du worktree (identiques à `29762b89a` pour ce fichier).
- **Écartées** : construire dans le worktree avant d'écrire le correctif (attente du verrou) ; un second worktree (interdit).
- **Réversible** : oui ; le dossier `avant-src/` reste dans `kesh-gate-logs/e36-5/` (aucun `rm`), à retirer par
  l'orchestrateur.

### C-36-5-11 — Le panneau testé avec le VRAI maintien, par délégation (développement)

- **Contexte** : AC5 bis (d) demande le vrai `authState` ; or le fichier simule `auth.svelte` pour tous ses tests (`logout`).
- **Retenu** : le simulacre d'`authState` garde `logout` simulé et délègue `holdForRestore`, `releaseRestoreHold` et
  `restoreHold` au vrai module, chargé en `beforeAll` par `vi.importActual` ; les espions comptent les appels, le vrai
  module tient l'échéance. Tous les tests du panneau voient donc le vrai maintien.
- **Écartées** : un faux maintien recopié dans le simulacre (une seconde implémentation à tenir) ; ne pas simuler
  `auth.svelte` (un simulacre partiel d'`api-client` qui importe le vrai `auth.svelte`, lequel importe `api-client` : cycle).
- **Réversible** : oui.

### C-36-5-12 — La phase 2 se reconnaît à la minuterie de lecture armée (développement)

- **Contexte** : D3 distingue au démontage la phase 2 (succès constaté, session connue morte) ; la fiche ne fixe pas le
  témoin.
- **Retenu** : `readingTimer !== null`. La minuterie n'est armée qu'au succès, si le panneau n'est pas démonté, et se remet
  à `null` en s'exécutant : un démontage après l'échéance de B (cas que V5-7 montre impossible) n'appellerait donc pas
  `expireSession()` une seconde fois, sans garde ajoutée. M19, M20 et M22 le gardent.
- **Écartée** : `imported !== null` (vrai aussi après l'exécution de la minuterie).
- **Réversible** : oui.

### C-36-5-13 — L'E2E de D5 retient la réponse dans la page, non par `page.route` (développement)

- **Contexte** : D5 prescrit `page.route('**/api/v1/admin/full-import', …)`, `route.fetch()` puis `route.fulfill`. Exécuté,
  le serveur reçoit une archive illisible (400 « Could not find EOCD », `backend-avant-d5.log`), et encore avec
  `postData` repassé en octets (`backend-avant-d5-r2.log`) : la requête multipart rejouée par Playwright n'est pas celle du
  navigateur. Les deux premières versions du test rougissaient donc pour une autre raison que la KF-056 — la première
  sans contrôle qui le dise (`rouges.log`, annotations).
- **Retenu** : `window.fetch` enveloppé par `page.evaluate` avant l'import : la requête part intacte, la réponse arrivée du
  serveur attend une promesse que le test libère après le 401 du battement ; le statut de l'import (200) et celui du
  battement (401) sont contrôlés **avant** l'attente du compte rendu. Le reste de D5 est tel qu'écrit (1,1 s, retrait de
  `heartbeatAt`, mouvements, seconde partie à compteur `page.route` sur `/auth/activity`, sans corps binaire).
- **Écartées** : `route.fetch()` (corps altéré) ; `page.clock` (déjà écartée par la fiche).
- **Réversible** : oui.
