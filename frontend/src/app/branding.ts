import packageJson from "../../../package.json";

export const APP_BASE_NAME = "Inventory Management";
export const APP_VERSION = packageJson.version;
export const APP_AUTHOR = "Syed Hassaan Shah";
export const APP_CREDIT = `Built by ${APP_AUTHOR}`;
export const APP_DISPLAY_NAME = `${APP_BASE_NAME} v${APP_VERSION}`;

/** Vite dev server builds (`tauri dev`). The backend of those builds uses isolated dev data. */
export const IS_DEV_BUILD = import.meta.env.DEV;
export const DEV_BUILD_TAG = "DEV build";
/** Footer corner: the author credit in releases, a DEV tag in developer builds. */
export const APP_FOOTER_TAG = IS_DEV_BUILD ? DEV_BUILD_TAG : APP_CREDIT;
