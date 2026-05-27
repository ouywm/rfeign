import { getCustomMDXComponent as getOriginalMDXComponent, Layout as BasicLayout } from '@rspress/core/theme-original';
import type { ComponentProps } from 'react';

import { LocalizedReadingTime } from './components/LocalizedReadingTime';
import './styles/reading-time.css';

function getCustomMDXComponent() {
  const { h1: H1, ...mdxComponents } = getOriginalMDXComponent();

  function HeadingWithReadingTime(props: ComponentProps<'h1'>) {
    return (
      <>
        <H1 {...props} />
        <LocalizedReadingTime />
      </>
    );
  }

  return {
    ...mdxComponents,
    h1: HeadingWithReadingTime,
  };
}

const Layout = () => <BasicLayout />;

export { getCustomMDXComponent, Layout };
export * from '@rspress/core/theme-original';
