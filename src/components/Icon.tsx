// Ícones de UI do design system (assets/ui-icons, SVG com currentColor).
import checkCircle from "../assets/icons/check-circle.svg?raw";
import chevronUpdown from "../assets/icons/chevron-updown.svg?raw";
import check from "../assets/icons/check.svg?raw";
import clipboard from "../assets/icons/clipboard.svg?raw";
import download from "../assets/icons/download.svg?raw";
import exclamation from "../assets/icons/exclamation.svg?raw";
import film from "../assets/icons/film.svg?raw";
import folder from "../assets/icons/folder.svg?raw";
import gear from "../assets/icons/gear.svg?raw";
import link from "../assets/icons/link.svg?raw";
import musicNote from "../assets/icons/music-note.svg?raw";
import magnifier from "../assets/icons/magnifier.svg?raw";
import play from "../assets/icons/play.svg?raw";
import retry from "../assets/icons/retry.svg?raw";
import trash from "../assets/icons/trash.svg?raw";
import trayDownload from "../assets/icons/tray-download.svg?raw";
import xmark from "../assets/icons/xmark.svg?raw";

const icons = {
  "check-circle": checkCircle,
  check,
  "chevron-updown": chevronUpdown,
  clipboard,
  download,
  exclamation,
  film,
  folder,
  gear,
  link,
  magnifier,
  "music-note": musicNote,
  play,
  retry,
  trash,
  "tray-download": trayDownload,
  xmark,
} as const;

export type IconName = keyof typeof icons;

export function Icon({ name, className }: { name: IconName; className?: string }) {
  return (
    <span
      className={className ? `icon ${className}` : "icon"}
      aria-hidden="true"
      dangerouslySetInnerHTML={{ __html: icons[name] }}
    />
  );
}
