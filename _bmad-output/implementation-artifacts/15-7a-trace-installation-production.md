# Story 15.7a : La piste de contrôle de l'installation de production

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni tests.** Elle ne garde que
les pointeurs vers ses deux moitiés et la trace de la passe qui a conduit au découpage. *(La
définition du statut `split` l'impose ; précédents : 15-7, 15-1, 15-5.)* La version complète d'avant
découpage se lit au commit `3846b206`.

## Les deux sous-stories

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-7a1** | `15-7a1-socle-transactions-onboarding.md` | le **socle** `kesh-db` : variantes `_in_tx` (`accounts`, `bank_accounts`, `company_invoice_settings`, `vat_rates`), `companies::clear_stub_in_tx`, `onboarding::lock_state_in_tx` — **sans changement de comportement ni d'audit** | `refs #434` |
| **15-7a2** | `15-7a2-trace-installation-production.md` | les **neuf routes de production** : une transaction par route, `lock_state_at_step`, helper d'étape, entrées d'audit, quatre actions et leurs libellés, registre (94 → 103), manuel, CHANGELOG | `refs #434` |

Ordre : 15-7a1 → 15-7a2 → 15-7b. La 15-7b ferme #434, #528 et #279.

## Pourquoi le découpage

Les deux lentilles de la passe P2 (R2-3, F-1) ont relevé qu'à la granularité que C-15-7-8 avait
retenue pour découper la 15-7, la 15-7a touchait encore **neuf** modules — six repositories `kesh-db`,
les routes d'onboarding, les libellés d'audit, `kesh-i18n` — sans section de dérogation. La seule
dérogation que le `CLAUDE.md` codifie est le cycle de dépendance Cargo, absent ici. L'orchestrateur a
appliqué le patron « story-zéro + rollout » (choix **C-15-7-19**) : les extractions mécaniques d'abord,
les routes et l'audit ensuite. Les remédiations de la passe P2 ont été appliquées **dans les deux
fiches filles**, pas ici ; leur tableau est au Change Log de la 15-7a2.

## Change Log

- 2026-10-08 — Née du découpage de la 15-7 à la passe P1 (C-15-7-8) ; 14 AC, 8 tâches, 12 tests.
- 2026-10-08 — Passe de validation P2 (prompt `15-7a-validate-prompt-p2.md`) : 0 CRITICAL, 0 HIGH,
  5 MEDIUM, 15 LOW distincts ; **découpée** en 15-7a1 + 15-7a2 (C-15-7-19). Corps vidé.
