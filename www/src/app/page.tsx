'use client';
import {useState} from 'react';
import {Graphviz} from 'graphviz-react';

export default function Home() {

  const [dotStr, setDotStr] = useState<string>("");

  const submitHandler = async () => {
    const rregex = (await import('rregex'));
    try {
      const enfa = rregex.get_enfa_from_regex("b*");
      const nfa = enfa.convert_to_nfa();
      const dfa = nfa.get_minimized_dfa();
      const post = dfa.to_fa_rep();

      setDotStr(post.get_dot_str());
      console.log(post.get_dot_str());
      console.log(post.get_start());
      console.log(post.get_fin());
      post.free();
      enfa.free();
    } catch (err) {
      console.error("Error while converting regex", err);
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
      {dotStr && <Graphviz dot={dotStr} />}
    </main>
  );
}
