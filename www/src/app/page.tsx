'use client';
import {useState} from 'react';
import dynamic from 'next/dynamic';
import {makeDotStrProper} from '@/utils/graphviz';
const Graphviz = dynamic(() => import('graphviz-react'), {ssr: false});

export default function Home() {
  const [dotStr, setDotStr] = useState<string>('');

  const submitHandler = async () => {
    const rregex = await import('rregex');
    try {
      const enfa = rregex.get_enfa_from_regex('ab|(b*f|e)*cd');
      const nfa = enfa.convert_to_nfa();
      const dfa = nfa.get_minimized_dfa();
      const post = dfa.to_fa_rep();
      const localDotStr = makeDotStrProper(
        post.get_dot_str(),
        post.get_start(),
        Array.from(post.get_fin())
      );

      setDotStr(localDotStr);
      console.log(localDotStr);
      console.log(post.get_start());
      console.log(post.get_fin());
      post.free();
      enfa.free();
    } catch (err) {
      console.error('Error while converting regex', err);
    }
    rregex.greet();
  };

  return (
    <main>
      <h1 className="text-3xl">RRegex</h1>
      <p>
        <button
          onClick={() => {
            (async () => {
              await submitHandler();
            })().catch((e: unknown) => {
              console.error(`Could not submit: ${e}`);
            });
          }}
        >
          Click here
        </button>
      </p>
      {dotStr && (
        <Graphviz
          dot={dotStr}
          options={{
            width: (5 / 6) * window.innerWidth,
          }}
        />
      )}
    </main>
  );
}
