import { InfiniteMovingCards } from "./ui/infinite-moving-cards";

import { useCases } from "@/app/data";

export const UseCases = () => {
  return (
    <section className="mt-36" id="use-cases">
      <h2 className="text-4xl font-bold text-center">Use Cases</h2>
      <h3 className="text-lg text-gray-400 text-center mb-10">
        Here are some of the ways you can use our product
      </h3>
      <InfiniteMovingCards direction="right" items={useCases} speed="slow" />
    </section>
  );
};
