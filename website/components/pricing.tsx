"use client";

import { Button } from "@nextui-org/react";
import { useState } from "react";
import Link from "next/link";

import { cn } from "@/lib/utils";
import { pricingPlans } from "@/app/data";

export const Pricing = () => {
  const plans = pricingPlans.map((category) => category.items);
  const [activeCategory, setActiveCategory] = useState(
    pricingPlans[0].category,
  );

  if (plans.length < 1) {
    return <h2>No</h2>;
  }

  return (
    <section
      className="max-w-6xl px-4 py-8 mx-auto sm:py-24 sm:px-6 lg:px-8"
      id="pricing"
    >
      <div className="sm:flex sm:flex-col sm:align-center">
        <h2 className="text-4xl font-bold text-center mt-16 mb-2 text-indigo-400">
          Hook0 Pricing
        </h2>
        <p className="max-w-2xl m-auto text-xl text-zinc-200 sm:text-center sm:text-2xl">
          Choose the plan that&apos;s right for you, and start sending webhooks
          now
        </p>
      </div>
      <div className="flex space-x-4 justify-center mt-6">
        {pricingPlans.map((category) => (
          <Button
            key={category.category}
            className={`text-sm font-semibold text-center rounded-md hover:bg-indigo-800 ${category.category === activeCategory ? "bg-indigo-800 text-white" : "bg-white text-indigo-400"}`}
            type="button"
            variant="shadow"
            onClick={() => setActiveCategory(category.category)}
          >
            {category.category}
          </Button>
        ))}
      </div>
      <div className="mt-12 space-y-0 sm:mt-16 flex flex-wrap justify-center gap-6 lg:max-w-4xl lg:mx-auto xl:max-w-none xl:mx-0">
        {pricingPlans
          .find((category) => category.category === activeCategory)
          ?.items.map((item) => (
            <div
              key={item.name}
              className={cn(
                "flex flex-col rounded-lg shadow-sm divide-y divide-zinc-600 bg-zinc-900",
                "flex-1",
                "basis-1/4",
              )}
            >
              <div className="p-6">
                <h2 className="text-2xl font-semibold leading-6 text-white">
                  {item.name}
                </h2>
                <p className="mt-4 text-zinc-300">{item.description}</p>
                <p className="mt-8">
                  <span className="text-5xl font-extrabold white">
                    {item.price}
                  </span>
                  <span className="text-base font-medium text-zinc-100">
                    /{item.duration}
                  </span>
                </p>
                {item?.second_price && item?.second_duration && (
                  <div className="mt-2">
                    <span className="text-xl font-bold text-zinc-100 ml-12">
                      +
                    </span>
                    <p className="ml-4 mt-2">
                      <span className="text-xl font-medium text-zinc-100">
                        {item.second_price}
                      </span>
                      <span className="text-base font-medium text-zinc-100">
                        /{item.second_duration}
                      </span>
                    </p>
                  </div>
                )}
                <Link href={item.link}>
                  <Button
                    className="block w-full py-2 mt-8 text-sm font-semibold text-center text-white rounded-md hover:bg-indigo-800"
                    variant="shadow"
                  >
                    {item.link_text}
                  </Button>
                </Link>
                <hr className="my-6" />
                <div className="space-y-2">
                  {item.includes.map((include) => (
                    <div key={include.text} className="flex items-center">
                      <include.icon
                        className={`text-xl ${include.status === "success" ? "text-green-400" : include.status === "warning" ? "text-orange-400" : "text-red-400"}`}
                      />
                      <span className="ml-2">{include.text}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ))}
      </div>
    </section>
  );
};
