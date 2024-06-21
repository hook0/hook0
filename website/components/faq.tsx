"use client";
import React from "react";
import { Accordion, AccordionItem } from "@nextui-org/react";

import { faq } from "@/app/data";

export const Faq = () => {
  return (
    <section className="mt-24" id="faq">
      <h2 className="text-4xl font-bold text-center mb-4 text-indigo-600">
        Frequently asked questions
      </h2>
      <p className="max-w-2xl mb-10 m-auto text-xl text-zinc-200 sm:text-center sm:text-2xl">
        Can’t find the answer you’re looking for? Reach out to our{" "}
        <a className="text-indigo-400" href="mailto:support@hook0.com">
          customer support
        </a>{" "}
        team.
      </p>
      <Accordion variant="light">
        {faq.map((item, index) => (
          <AccordionItem key={index} title={item.question}>
            {item.answer}
          </AccordionItem>
        ))}
      </Accordion>
      <p className="max-w-2xl mt-8 m-auto text-xl text-zinc-200 sm:text-center sm:text-xl">
        Have more questions?
      </p>
      <p className="max-w-2xl m-auto text-xl text-zinc-200 sm:text-center sm:text-lg">
        Check out our{" "}
        <a
          className="text-indigo-400"
          href="https://documentation.hook0.com/docs/getting-started"
        >
          product FAQ
        </a>
        ,{" "}
        <a
          className="text-indigo-400"
          href="https://documentation.hook0.com/docs/pricing-plans-faq"
        >
          pricing FAQ
        </a>
        , or{" "}
        <a className="text-indigo-400" href="mailto:support@hook0.com">
          contact sales team
        </a>
        .
      </p>
    </section>
  );
};
