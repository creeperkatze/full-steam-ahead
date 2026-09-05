<script setup lang="ts">
import { computed } from 'vue'

import AmazonIcon from '../assets/icons/amazon.svg'
import BottlesIcon from '../assets/icons/bottles.svg?url'
import EpicIcon from '../assets/icons/epic.svg'
import FlatpakIcon from '../assets/icons/flatpak.svg?url'
import GamePassIcon from '../assets/icons/gamepass.svg'
import GogIcon from '../assets/icons/gog.svg'
import HeroicIcon from '../assets/icons/heroic.svg?url'
import ItchIcon from '../assets/icons/itch.svg'
import LegendaryIcon from '../assets/icons/legendary.png'
import LutrisIcon from '../assets/icons/lutris.svg?url'
import MinigalaxyIcon from '../assets/icons/minigalaxy.png'
import OriginIcon from '../assets/icons/origin.svg'
import PlayniteIcon from '../assets/icons/playnite.svg'
import UbisoftIcon from '../assets/icons/ubisoft.svg'

const ICONS: Record<string, unknown> = {
	amazon: AmazonIcon,
	bottles: BottlesIcon,
	epic: EpicIcon,
	flatpak: FlatpakIcon,
	gamePass: GamePassIcon,
	gog: GogIcon,
	heroic: HeroicIcon,
	itch: ItchIcon,
	legendary: LegendaryIcon,
	lutris: LutrisIcon,
	miniGalaxy: MinigalaxyIcon,
	origin: OriginIcon,
	playnite: PlayniteIcon,
	ubisoftConnect: UbisoftIcon,
}

const props = defineProps<{ source: string }>()
const icon = computed(() => ICONS[props.source])
</script>

<template>
	<img v-if="typeof icon === 'string'" :src="icon" alt="" />
	<component :is="icon" v-else-if="icon" />
</template>
