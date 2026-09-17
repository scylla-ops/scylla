import { useLocation, useOutlet } from 'react-router-dom';

//TODO: make the animation launchable from a children
// and when the loading is finished, start the animation
export const AnimatedOutlet = () => {
  const location = useLocation();
  const element = useOutlet();

  // `key` is what replays the animation: a new pathname remounts the node, and
  // the enter utilities run again. There is no leave animation — CSS cannot
  // animate a node React has already unmounted, and the 200ms fade-out this
  // used to do only held the next page back.
  return (
    <main
      key={location.pathname}
      className='flex flex-col h-full w-full p-2 animate-in fade-in-0 zoom-in-[0.98] duration-200 ease-in-out'
    >
      {element}
    </main>
  );
};
