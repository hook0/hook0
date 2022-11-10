import { intersectWith } from './fp';
import { head } from 'ramda';

describe('intersectWith', function () {
  test('works', () => {
    type TypeInput = {
      attr: string;
    };

    type TypeOutput = TypeInput & {
      selected: boolean;
    };

    const res = intersectWith<TypeInput, TypeOutput, string>(
      (a) => a.attr,

      (list) => {
        return {
          ...(head(list) as TypeInput),
          selected: true,
        };
      },

      [{ attr: 'a' }, { attr: 'b' }],
      [{ attr: 'b' }, { attr: 'c' }, { attr: 'd' }]
    );

    expect(res).toMatchInlineSnapshot(`
      [
        {
          "attr": "a",
          "selected": true,
        },
        {
          "attr": "b",
          "selected": true,
        },
        {
          "attr": "c",
          "selected": true,
        },
        {
          "attr": "d",
          "selected": true,
        },
      ]
    `);
  });
});
