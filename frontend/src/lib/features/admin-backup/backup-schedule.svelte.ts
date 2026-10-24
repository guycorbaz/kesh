// Story 31-1c (#503, AC31) — l'état de la sauvegarde automatique, partagé entre
// le panneau d'administration et le bandeau du layout : une action du panneau
// (enregistrer, « Sauvegarder maintenant ») le met à jour, et le bandeau suit
// sans attendre son prochain sondage (R9 de la validation P5).
//
// Story 32-1b (#711 E-8) : un échec de lecture GARDE le dernier état lu (un
// sondage manqué n'éteint plus un bandeau d'échec), et une réponse tardive —
// arrivée après un état plus récent, posé par le panneau ou par une lecture
// lancée ensuite — est ignorée (compteur de séquence, motif de `fiscal-years`,
// C-29-1-37). Tant qu'aucune lecture n'a réussi, l'état reste `null`.
//
// Pattern objet avec getter (l'export direct d'un `$state` n'est pas
// réassignable depuis un importeur — cf. `auth.svelte.ts`).

import { fetchBackupSchedule, type BackupSchedule } from './backup-schedule.api';

let _value = $state<BackupSchedule | null>(null);

/**
 * Numéro de la dernière écriture demandée : incrémenté par chaque
 * `setBackupScheduleState` et au départ de chaque lecture. Une lecture dont le
 * numéro n'est plus le dernier à son retour est ignorée. Non réactif.
 */
let _sequence = 0;

export const backupScheduleState = {
	/**
	 * Le dernier état connu ; `null` tant qu'aucune lecture n'a réussi (un échec
	 * de lecture ultérieur garde l'état précédent).
	 */
	get value(): BackupSchedule | null {
		return _value;
	},
};

/** Pose l'état rendu par une action du panneau ; toute lecture en vol est périmée. */
export function setBackupScheduleState(value: BackupSchedule | null): void {
	_sequence += 1;
	_value = value;
}

/**
 * Relit l'état. Un échec de lecture **garde** l'état précédent — `null` si
 * aucune lecture n'a encore réussi, et le bandeau ne s'affiche alors pas
 * (AC31 : un échec de cette lecture n'affiche aucun bandeau et ne casse pas
 * le layout, pas de toast). Une réponse arrivée après un état plus récent est
 * ignorée.
 */
export async function refreshBackupScheduleState(): Promise<void> {
	_sequence += 1;
	const sequence = _sequence;
	try {
		const value = await fetchBackupSchedule();
		if (sequence === _sequence) _value = value;
	} catch {
		// Le dernier état lu reste affiché (Story 32-1b, #711 E-8).
	}
}
