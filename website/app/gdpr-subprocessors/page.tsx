"use client";
import React from "react";
import {
  Table,
  TableHeader,
  TableColumn,
  TableBody,
  TableRow,
  TableCell,
  getKeyValue,
} from "@nextui-org/react";

export default function GdprSubprocessorsPage() {
  const columns = [
    { key: "subprocessor", label: "Subprocessor" },
    { key: "country", label: "Country" },
    { key: "purpose", label: "Purpose" },
  ];
  const rows_table_1 = [
    {
      key: "1",
      subprocessor: "CleverCloud SAS",
      country: "France, Europe",
      purpose: "Hook0 customer database, API and web application",
    },
  ];
  const rows_table_2 = [
    {
      key: "1",
      subprocessor: "CleverCloud SAS",
      country: "France, Europe",
      purpose: "Workers that calls the webhook subscription endpoints",
    },
    {
      key: "2",
      subprocessor: "Stripe Inc.",
      country: "USA",
      purpose: "Hook0's customer subscription management",
    },
  ];
  return (
    <section id="gdpr-subprocessors">
      <div className="mt-40 relative py-16 overflow-hidden">
        <div className="relative px-4 sm:px-6 lg:px-8">
          <div className="text-lg max-w-prose prose prose-lg mx-auto">
            <h1>
              <span className="block text-center text-indigo-500 text-2xl font-bold tracking-wide uppercase mb-10">
                What are your sub-processors, as defined by the GDPR?
              </span>
            </h1>

            <div>
              <div className="section haze wf-section">
                <div className="wrapper">
                  <div className="intro">
                  <div className="mb-8 text-xl font-semibold">
                      Last Update: November 14th, 2021
                    </div>
                    <div className="divider"></div>
                  </div>
                  <div className="w-layout-grid main-grid">
                    <div className="text-slate-300">
                      <p className="mb-4">
                        Hook0 uses certain sub-processors to assist it in
                        providing to its customers the Application Services as
                        described in the Master Services Agreement or Terms of
                        Use available at{" "}
                        <a href="/terms" className="text-indigo-500 underline">terms-of-service</a> or such other
                        location as the Terms of Use may be posted from time to
                        time (as applicable, the “Agreement”). Defined terms
                        used herein shall have the same meaning as defined in
                        the Agreement.
                      </p>
                      <p className="mb-4">
                        A subprocessor is a third party data processor engaged
                        by Hook0, including entities from within the Hook0
                        group, who has or potentially will have access to or
                        process Customer Content (which may contain Personal
                        Data). Hook0 engages different types of subprocessors to
                        perform various functions as explained in the tables
                        below.
                      </p>
                      <p className="mb-6">
                        Infrastructure: We use the following subprocessors to
                        provide our cloud infrastructure environment and storage
                        of our Customer Content:
                      </p>
                      <Table aria-label="Hook0 subprocessors table" className="mb-6">
                        <TableHeader columns={columns}>
                          {(column) => (
                            <TableColumn key={column.key}>
                              {column.label}
                            </TableColumn>
                          )}
                        </TableHeader>
                        <TableBody items={rows_table_1}>
                          {(item) => (
                            <TableRow key={item.key}>
                              {(columnKey) => (
                                <TableCell>
                                  {getKeyValue(item, columnKey)}
                                </TableCell>
                              )}
                            </TableRow>
                          )}
                        </TableBody>
                      </Table>

                      <p className="mb-4">
                        Processing of Customer Content: We work with various
                        subprocessors that monitor, maintain and otherwise
                        support the Application Services. In order to provide
                        this functionality these subprocessors may, but not
                        necessarily will, have access to Customer Content:
                      </p>
                      <p className="mb-6">
                        * Note, the list of subprocessors applies to any new
                        Hook0 customers as of that date, or existing Hook0
                        customers who have not otherwise received notice of a
                        different effective date of this list.
                      </p>
                      <Table aria-label="Hook0 subprocessors table" className="mb-6">
                        <TableHeader columns={columns}>
                          {(column) => (
                            <TableColumn key={column.key}>
                              {column.label}
                            </TableColumn>
                          )}
                        </TableHeader>
                        <TableBody items={rows_table_2}>
                          {(item) => (
                            <TableRow key={item.key}>
                              {(columnKey) => (
                                <TableCell>
                                  {getKeyValue(item, columnKey)}
                                </TableCell>
                              )}
                            </TableRow>
                          )}
                        </TableBody>
                      </Table>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
