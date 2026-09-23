import { getContext, setContext } from 'svelte';
import { MediaQuery } from 'svelte/reactivity';

const SIDEBAR_KEY = Symbol('sidebar');
const SIDEBAR_COOKIE_NAME = 'sidebar_state';
const SIDEBAR_COOKIE_MAX_AGE = 60 * 60 * 24 * 7;
const MOBILE_QUERY = 'max-width: 767px';

/** The open state of the sidebar, shared by every sidebar part below the provider. */
export interface SidebarState {
  readonly open: boolean;
  openMobile: boolean;
  readonly isMobile: boolean;
  readonly state: 'expanded' | 'collapsed';
  setOpen: (open: boolean) => void;
  toggle: () => void;
}

const createSidebarState = (): SidebarState => {
  let open = $state(true);
  let openMobile = $state(false);
  const mobile = new MediaQuery(MOBILE_QUERY);

  const setOpen = (value: boolean) => {
    open = value;
    document.cookie = `${SIDEBAR_COOKIE_NAME}=${value}; path=/; max-age=${SIDEBAR_COOKIE_MAX_AGE}`;
  };

  return {
    get open() {
      return open;
    },
    get openMobile() {
      return openMobile;
    },
    set openMobile(value: boolean) {
      openMobile = value;
    },
    get isMobile() {
      return mobile.current;
    },
    get state() {
      return open ? 'expanded' : 'collapsed';
    },
    setOpen,
    toggle: () => {
      if (mobile.current) openMobile = !openMobile;
      else setOpen(!open);
    },
  };
};

/** Creates the sidebar state and gives it to the components below. */
export const setSidebar = (): SidebarState => setContext(SIDEBAR_KEY, createSidebarState());

/** The sidebar state of the nearest `SidebarProvider`. */
export const getSidebar = (): SidebarState => {
  const sidebar = getContext<SidebarState | undefined>(SIDEBAR_KEY);
  if (!sidebar) throw new Error('A sidebar part must be inside a SidebarProvider.');
  return sidebar;
};
