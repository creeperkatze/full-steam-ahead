<script setup lang="ts">
import { computed } from "vue";
import AmazonIcon from "../assets/icons/amazon.svg";
import EpicIcon from "../assets/icons/epic.svg";
import GamePassIcon from "../assets/icons/game_pass.svg";
import GogIcon from "../assets/icons/gog.svg";
import ItchIcon from "../assets/icons/itch.svg";
import OriginIcon from "../assets/icons/origin.svg";
import PlayniteIcon from "../assets/icons/playnite.svg";
import UbisoftIcon from "../assets/icons/ubisoft.svg";

const ICONS: Record<string, unknown> = {
  amazon: AmazonIcon,
  epic: EpicIcon,
  gamePass: GamePassIcon,
  gog: GogIcon,
  itch: ItchIcon,
  origin: OriginIcon,
  playnite: PlayniteIcon,
  ubisoftConnect: UbisoftIcon,
};

const props = defineProps<{ source: string }>();
const icon = computed(() => ICONS[props.source]);
</script>

<template>
  <component :is="icon" v-if="icon" />
</template>
