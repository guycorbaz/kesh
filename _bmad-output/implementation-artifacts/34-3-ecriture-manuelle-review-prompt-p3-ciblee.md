# Revue de code P3 ciblée — 34-3 « Écriture manuelle » (#967), une lentille (lecture seule)

Tu es la **passe ciblée P3** de la boucle `bmad-code-review` de la story **34-3** de Kesh. La passe ciblée P2, sur le
commit de remédiation `584a1183`, a rendu 1 MEDIUM et 1 LOW (`/home/gcorbaz/devel/kesh-gate-logs/e3403-review-p2-ciblee.md`) :

- **C2-1** : la mutation M2 n'avait pas été rejouée sur l'E2E ;
- **C2-2** : la FAQ fournisseur du manuel gardait la formulation que C-e3403-8 déclare ambiguë.

Tu relis ce qui y répond, et seulement cela. Contexte frais. Travaille en français soigné.

## Périmètre

1. **Le commit C2-2** :
   ```sh
   git -C /home/gcorbaz/devel/kesh-e3403 show f5969619
   ```
   C'est la FAQ fournisseur de `docs/manual/fr/user-manual.tex`. Elle doit maintenant dire le même sens que l'avis
   créanciers. Ce sens est fixé par C-e3403-8 dans `_bmad-output/implementation-artifacts/e34-choix-autonomes-e3403.md` et
   repris dans le catalogue fr, clé `reconciliation-manual-claim-notice-payable` : « saisissez la facture et validez-la
   depuis sa fiche, sans y enregistrer de paiement : le paiement est cette transaction ». Vérifie trois points :
   - la phrase est correcte, et ses renvois `\S\ref` sont intacts ;
   - aucune autre ligne du manuel ne garde l'ancienne formule ;
   - la phrase suivante (« \emph{Accepter} la règle ») reste cohérente.

2. **Les journaux des gates différés et de M2 sur l'E2E**, joués au commit `f5969619` après la levée de la pause :
   - `/home/gcorbaz/devel/kesh-gate-logs/e3403/p1-reprise.log` (jalons `check=`, `vitest=`, `build-M2=`, `build=`,
     `e2e-fait`) ;
   - `/home/gcorbaz/devel/kesh-gate-logs/e3403/p1/check.log`, `vitest.log`, `build.log`, `e2e-complet.log` ;
   - `/home/gcorbaz/devel/kesh-gate-logs/e3403/mut/M2-e2e.log` et `M2-e2e-run.log`.

   Pour M2, vérifie trois points :
   - la mutation est-elle la bonne ? (`mut-e3403.py` : retrait de `a.id !== bankLedgerAccountId`) ;
   - le rouge vient-il de l'**ancre positive** de `frontend/tests/e2e/reconciliation-manual-entry.spec.ts`, l'option du
     compte de la banque présente après la saisie de `1`, et non d'une autre assertion ou d'un problème de montage ?
   - le fichier a-t-il été restauré, puis le frontend reconstruit avant l'E2E complet ?

   Pour l'E2E complet, chaque échec doit être nommé contre `docs/testing.md` § *Les échecs attendus* : les sept de la
   KF-029, et #994 si `KESH_E2E_CAPTURES_DIR` était posé. Le spec `reconciliation-manual-entry.spec.ts` doit être vert.

3. **Le Change Log de la fiche** (`34-3-ecriture-manuelle.md`, dernière entrée) : les chiffres déclarés doivent égaler
   ceux des journaux. Recompte-les depuis les journaux.

## ⛔ Ground-truth obligatoire

Toute affirmation s'appuie sur une commande `/usr/bin/grep -nF "<chaîne exacte>" <fichier>`, dont tu copies la sortie.
Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep`.

## Sévérités

- **CRITICAL** : écriture fausse possible.
- **HIGH** : gate déclaré vert qui ne l'est pas, mutation non tuée.
- **MEDIUM** : chiffre déclaré faux, mutation tuée pour une mauvaise raison, manuel faux.
- **LOW** : rédaction.

## Interdits — lecture seule stricte

Aucune commande qui écrit, compile ou exécute :
- pas de `cargo`, `npm`, `npx`, `make`, `latexmk`, `docker`, `curl`, `sqlx` ni `mariadb` ;
- **pas de `scripts/prepare-release.sh`**, ni aucun script de `scripts/` ou de `kesh-gate-logs/` ;
- pas de `git checkout`, `switch`, `reset`, `stash`, `commit`, `push`, `tag` ni `rebase` ;
- aucune commande `gh` qui écrit ;
- pas de `claude`, `pkill`, `killall`, `rm`, `sed -i`, `perl -pi` ni `tee` ;
- **aucune redirection `>`, aucun fichier temporaire**.

Tu n'écris qu'un seul fichier : ton rapport.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e3403-review-p3-ciblee.md` (jamais sous `target/`) :

1. **Décompte** CRITICAL / HIGH / MEDIUM / LOW, recompté depuis ta liste.
2. **Findings** : identifiant (`C3-1`…), sévérité, fichier et ligne, sortie `grep -nF` copiée, correctif proposé.
3. **Axes exercés et non exercés** : liste explicite des points 1 à 3. Un « 0 finding » sans cette liste ne compte pas.
