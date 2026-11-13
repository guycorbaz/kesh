# Epic 35 — registre des choix autonomes de la story 35-4 (clé `e354`)

Registre propre à la story 35-4 (*Rejeter* une ligne signalée demande confirmation ; une relecture referme la
confirmation ; #1023, #1024), **versé par union** par l'orchestrateur. Chaque choix : contexte, option retenue, options
écartées, réversibilité.

---

### Constat de l'étape 0 — la liste ne se relit pas sous une confirmation ouverte (2026-11-12)

- **Protocole** : E2E `frontend/tests/e2e/reconciliation-already-booked-reject.spec.ts`, second test, joué dans Chromium
  (Playwright) sur le binaire et le frontend de **départ** (`1ebe1240`, = la 0.17.6 publiée + les fiches de l'Epic 35 ;
  aucun fichier de `src/` modifié), backend `127.0.0.1:3404`, base jetable `kesh_e2e_e354`. Montage : une écriture
  `D charge / C banque` d'un montant unique, un relevé saisi dont le débit l'égale (ligne signalée), puis un relevé CAMT
  synthétique de 101 transactions plus anciennes : la liste en montre cent et le bouton *Afficher aussi les plus
  anciennes* est rendu. Confirmation ouverte par *Écriture manuelle* sur la ligne signalée.
- **Constat** (journal `kesh-gate-logs/e354/e2e-r0/playwright.log`, test vert) :
  - `reconciliation-show-older-btn.click({ timeout: 2000 })` échoue sur « intercepts pointer events » : le bouton ne
    reçoit pas le clic ;
  - six pressions de Tab : le focus reste chaque fois dans le dialogue ;
  - aucune requête `…/reconciliation/proposals` ne part pendant que la confirmation est ouverte ; la liste garde ses cent
    lignes ;
  - Échap, puis le clic hors du dialogue (`mouse.click(5, 5)`), referment la confirmation sans rien ouvrir.
- **Conclusion** : le constat **confirme** celui de la fiche (D5) ; le chemin de #1024 (une relecture sous une
  confirmation ouverte) n'était atteignable qu'en test (jsdom). Le commentaire de fermeture de D5 peut être posté tel
  quel, la phrase entre crochets remplie par ce constat.

### C-e354-1 — Le constat de l'étape 0 devient un test E2E permanent ; le relevé synthétique est partagé

- **Contexte** : l'étape 0 demande un constat en navigateur, consigné ici. Le jouer par un script hors du dépôt l'aurait
  rendu invérifiable après coup ; le jouer par une spec non versionnée aurait laissé un fichier qu'aucune consigne ne
  permet d'effacer (pas de `rm`).
- **Retenu** : le constat est le second test de la spec neuve de AC7 (`reconciliation-already-booked-reject.spec.ts`) :
  il reste vrai après la story (la modale bloque toujours la page) et garde la conclusion de D5 sous test — si un jour la
  page se relit sous une confirmation, il rougit. Le relevé CAMT synthétique de `reconciliation-show-older.spec.ts` est
  extrait dans `frontend/tests/e2e/helpers/camt-synthetic.ts` (règle DRY du `CLAUDE.md`), paramétré par le nombre
  d'entrées et un marqueur ; la spec de la 29-9 l'importe, inchangée sur le fond (mêmes références `OLDER-…`, mêmes
  contreparties `Ancienne …`).
- **Écartées** : (a) script Playwright hors du dépôt (`kesh-gate-logs/`) : `@playwright/test` ne s'y résout pas, et le
  constat ne serait pas rejoué ; (b) recopier le constructeur du relevé dans la spec neuve : trente lignes dupliquées.
- **Coût** : une spec de plus à la suite E2E (≈ 5 s, l'import de 101 transactions compris).
- **Réversible** : oui — retirer le second test et réintégrer le constructeur dans la spec de la 29-9.

### C-e354-2 — Le « clic hors » de l'E2E est réessayé jusqu'à la fermeture

- **Contexte** : à l'E2E complet (`e2e-r2`), le premier test de la spec neuve a rougi sur
  `page.mouse.click(5, 5)` puis `toHaveCount(0)` : la confirmation restait ouverte. Rejoué seul trois fois (`e2e-r3`),
  il rougit deux fois sur trois ; le second test, même geste, passe. Lu dans `bits-ui`
  (`dist/bits/utilities/dismissible-layer/use-dismissable-layer.svelte.js`) : les écouteurs `pointerdown` du clic hors
  ne sont posés qu'après `afterSleep(1, …)` suivant le montage, et le clic est traité par un `debounce(…, 10)`. Le clic
  de Playwright, tiré dès que le dialogue est visible, peut précéder la pose des écouteurs.
- **Retenu** : un helper `cliquerHors` qui réessaie le clic (`expect(…).toPass`, 5 s) tant que la confirmation n'est
  pas refermée ; la fermeture reste exigée. Ce n'est pas un défaut du produit : aucun clic humain ne tombe dans la
  fenêtre de quelques millisecondes qui suit l'ouverture.
- **Écartées** : (a) `waitForTimeout` fixe avant le clic : délai arbitraire, aussi fragile sous charge ; (b) retirer le
  clic hors de l'E2E : AC1 renvoie son constat à l'étape 0 ou à l'E2E, aucun test jsdom ne le simulant.
- **Risque** : si un clic réessayé tombait après une fermeture lente, il atteindrait l'en-tête (lien *Kesh*) et la
  suite du test rougirait — un échec bruyant, jamais un vert à tort.
- **Réversible** : oui.
