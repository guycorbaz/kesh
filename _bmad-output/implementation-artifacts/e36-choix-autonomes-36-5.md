# Epic 36 — registre des choix autonomes de la story 36-5 (`36-5-restauration-compte-rendu`, #931)

Choix faits à la rédaction de la fiche (2026-11-14, Opus 5.5, worktree `kesh-36-5`, code lu sur `4de0bcc7e`), dans le cadre
de C-e360-8 (« forme exacte à la fiche »). Chaque choix : contexte, option retenue, options écartées, réversibilité.

### C-36-5-1 — Un maintien de session porté par `authState`, lu par le chemin 401 du client

- **Contexte** : l'inventaire par le symptôme (fiche, § *Inventaire*) trouve deux renvois à la connexion atteignables après
  l'import sans geste de l'utilisateur : le chemin 401 du client (`api-client.ts:202`, nourri par le battement et, après
  l'`exp` du jeton d'accès, par le bandeau des sauvegardes) et la fermeture d'inactivité (`session-idle.svelte.ts:407`).
- **Retenu** : `authState.holdForRestore()` / `releaseRestoreHold()` / `restoreHold` ; sous maintien, un 401 jette
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
- **Retenu** : pendant le maintien, le module note l'activité et bat (le 401 éventuel est absorbé par C-36-5-1), mais ne ferme
  pas et n'avertit pas ; à la levée, le tic suivant ferme si l'échéance est passée. Au succès, le panneau arrête le module
  (`sessionIdle.stop()`) ; à l'issue inconnue, maintien gardé, module laissé tel quel ; à l'échec certain et au démontage du
  panneau, maintien levé.
- **Écartées** : suspendre aussi le battement ; lever le maintien dès la réponse reçue.
- **Réversible** : oui.
- **Amendé le 2026-11-14 (validation P1, V1-3 ; option prudente demandée par l'orchestrateur)** : le maintien est **borné**
  — il ne couvre que l'import en cours et la lecture du compte rendu, chacune au plus B = la durée d'inactivité réglée
  (60 min si inconnue) ; il est **levé à l'issue inconnue** (la fermeture ordinaire reprend) ; à B après un succès, la
  session se termine comme par le chemin 401 (`clearSession`, `/login?reason=session_expired`). Levée en `finally`
  (V1-9). Écartées : maintien sans borne après une issue inconnue ; borne fixe de 15 min sans rapport avec le réglage.

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

### C-36-5-5 — Manuel inchangé ; la note de la KF-056 dans `docs/testing.md`

- **Contexte** : `admin-manual.tex:2340` décrit déjà le comportement rétabli (« Prenez le temps de lire, puis cliquez « Se
  reconnecter » ») ; `docs/testing.md` ne nomme pas la KF-056, tolérée implicitement dans « 0 à 2 de pollution ».
- **Retenu** : aucun changement du manuel ni de son PDF ; une note historique « KF-056 corrigée » après celle de la KF-052.
- **Écartée** : une phrase de plus au manuel (rien de visible n'y change : le code rejoint le texte).
- **Réversible** : oui.

### C-36-5-6 — Noms de bases

- **Contexte** : la clé contient des tirets, que les noms de bases MariaDB n'admettent pas sans guillemets.
- **Retenu** : `kesh_e2e_e365` et `kesh_ci_e365` (patron `e357` de l'Epic 35), journaux sous `kesh-gate-logs/e36-5/`.
- **Réversible** : oui.
