import { InfiniteMovingCards } from "./ui/infinite-moving-cards";

import { outoftheBox } from "@/app/data";

export const OutOfTheBox = () => {
  return (
    <div>
      <h2 className="text-4xl font-bold text-center mt-16 mb-10 text-indigo-600">
        Out-Of-The-Box Webhooks
      </h2>
      <InfiniteMovingCards direction="left" items={outoftheBox} speed="slow" />
    </div>
  );
};
