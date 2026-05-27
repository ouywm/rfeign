import { useDark, useLang, usePageData } from '@rspress/core/runtime';
import type { ReadTimeResults } from 'reading-time';

function formatReadingTime(result: ReadTimeResults, lang: string) {
  if (lang.startsWith('en')) {
    return `Estimated reading time: ${result.minutes >= 1 ? `${Math.ceil(result.minutes)} minutes` : 'less than 1 minute'}`;
  }

  return `预计阅读时间: ${result.minutes >= 1 ? `${Math.ceil(result.minutes)} 分钟` : '小于 1 分钟'}`;
}

export function LocalizedReadingTime() {
  const pageData = usePageData();
  const lang = useLang();
  const dark = useDark();
  const pageReadingTime = pageData.page.readingTimeData as
    | ReadTimeResults
    | undefined;

  if (!pageReadingTime) {
    return null;
  }

  return (
    <span className="sa-reading-time" data-dark={String(dark)}>
      {formatReadingTime(pageReadingTime, lang)}
    </span>
  );
}
