# rust-json-str-redactor
`rust-json-str-redactor` is a useful helper function to redact a JSON string according to given key sequences, while keeping the original string length and sequence.

## Context
Here's the context, I was working with a useful tool called TLSNotary. It allows you to notarize any HTTPS request you made to any website, and selectively disclose part of the JSON response.

In order to do the selective disclosure, you will need to specify the ranges that are included in the disclosure.

In our case, we would like to dynamically calculate the ranges based on a bunch of key sequences, so that only values under the specified key sequences are kept, and all other values redacted, at the same time all the structures and keys should be kept.

For example, for a JSON string like this
`{"name":"Alice","age":30,"contacts":[{"type":"email","value":"alice@email.com"},{"type":"phone","value":"123-456-7890"}],"isActive":true}`

We want to include values of `name`, and also `contacts.type`

The expected output would be `[[0, 22], [24, 61], [78, 104], [118, 132], [136, 137]]` which would be all the ranges that should be kept, the rest of the ranges on the other hand, should be redacted.

If we visualize the result, it would look like the following
```
{
  "age": "<REDACTED>",
  "contacts": [
    {
      "type": "email",
      "value": "<REDACTED>"
    },
    {
      "type": "phone",
      "value": "<REDACTED>"
    }
  ],
  "isActive": "<REDACTED>",
  "name": "Alice"
}
```

After thinking for a few hours how I can accomplish this without dealing with the tedious and errorprone char by char string processing, I had come up with no good solutions. Then I thought about chatGPT. I know roughly how it should work, why not have it do the tedious work and help me implement it? So here we go on a several hour journey back and forth with chatGPT and finally producing the working script.

## Problem definition
I first tried to describe the problem, in a leetcode style, I was afraid that it wouldn't understand what exactly I wanted to do, but it did, reasonably well.

```
Please help me write a rust function, which takes in a stringified JSON object and a sequence of keys, only keep the values if they are at the place specified by the sequence of keys, and redact all other values, while keeping all the keys and all structural characters such as brackets and commas. Details below:
1. Takes a stringified JSON object as first input
2. Takes an array of strings as the second input
3. The output is an array of ranges, indicating the char ranges that should not be redacted

Here's an example input and output of the program:
Input1: {"a": {"b":"c","d":1,"e":[{"f":"g"}]}}
Input2: ["a", "d"]
Output: [[0, 11], [14, 31], [34, 38]]

Feel free to ask more clarification questions if the description or example doesn't make sense
```

## Initial analysis and first try
chatGPT analyzed the task and gave me a game plan

```
Okay, let's break this down step by step.

Given a stringified JSON, you want to produce an array of ranges that indicate which parts of the string should NOT be redacted, based on a sequence of keys.

To achieve this:

We need to parse the stringified JSON.
Traverse the parsed JSON to find the sequence of keys and identify the ranges of characters that should remain unredacted.
Return the array of these ranges.
Here's a plan:

Parse the stringified JSON.
Write a recursive function to traverse the parsed JSON. While traversing, we'll keep track of the current sequence of keys and the current position in the stringified JSON.
If the current sequence of keys matches the input sequence, we'll compute the range of the value and add it to our output array.
Return the output array of ranges.
Before we dive into coding, does the plan make sense to you?
```

I then told it that `I want not only the ranges of specified values will be returned, but also include all the keys and all the structural components such as brackets and commas, and only leave the other values behind`

On the first try, it used my original solution, which is parsing the object, and then lookup the keys in it.

https://github.com/chcharcharlie/rust-json-str-redactor/commit/7f2beac00af734c57788f6869b6ba4cb162f8c52

This approach would not work however, because once you parse the JSON into an object, a lot of information were lost such as the sequence of the keys, format of some numeric values etc, and the returned ranges would not be correct to the original string.

## Traversing char by char
I gave the feedback to chatGPT and it understood, the next version it came up with is one that traverses the string char by char and handles each separately.

https://github.com/chcharcharlie/rust-json-str-redactor/commit/8a879d582b19b4fe08e6f4eb9e209a4028792f98

The program runs fine, and for a while I thought he already got it done.

## Debugging
But then the struggle begins, the program would always miss something. It works for one test case, but once I tweak the test case ever so slightly, it breaks. It's as if there were endless of edge cases, just that they were all not so "edgy".

It didn't recognize which strings were keys and which were values and updated according to my prompt:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/23688f2897f5c825e129ba4c2577d2a27467ca51

Had problems with popping keys out of the stack and missing values:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/ffc0ab264f014728426dcc6ca822702b232e4efd

I asked it to connect and merge ranges at the end instead of sending a bunch of small ranges:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/7ea00765b3c11219d3fe8b326e6967508f0e42d9

Tried to handle missing non string values:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/51e5d71f8c5f34ce2100aeb5b90403dd979d8bb1

`Sometimes you find that even though he says he's updating the code, he's actually not.`

## Deeper human intervene
Over time I found out, for most of the cases, if I only describe the symptom and expected behavior, chatGPT will be unable to get it going further. It repeats the same apologies but then makes cosmetic changes or sometimes even make it worse.

`I had to examine the code carefully, including add some printed logs and let it know where I think is wrong.`

Then it continued on improving with my help.

Multiple tries to handle stach pushing correctly:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/baeb911517a1cb7464094e632944c3f5f939f63a
https://github.com/chcharcharlie/rust-json-str-redactor/commit/d7cb53be6a796d0d2c0849245121f92a2e744418
https://github.com/chcharcharlie/rust-json-str-redactor/commit/e8ecfdedea599e2293ae8675218164ff01d7f604

Introduced capture_all and brace/bracket_counts to deal with cases where entire subobject is included:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/b19d058f1e001350cc1cdf4a95ac0bb4f2d7cff8

Debug closing double quotes:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/2e6aec83c75937008ba7e8ab4670896c75602cd2

Deal with capture_all on reverse brackets/braces:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/e95ab6471d6e01e0c5d5f99563df38211280eceb

Endless back and force debugging with the `capture_all` handling...

https://github.com/chcharcharlie/rust-json-str-redactor/commit/9bf2d3e5536374861f18eb201a5f825722d6c081
https://github.com/chcharcharlie/rust-json-str-redactor/commit/8a1d81eb03f118e142d16dcb47b87801ae726656
https://github.com/chcharcharlie/rust-json-str-redactor/commit/175caf20f2e0a8076ab3c0ff223cac1931b2ab87
https://github.com/chcharcharlie/rust-json-str-redactor/commit/e871d936f8ca7cf3ad5a1d0cb3383e164779f308

Handle `in_string` correctly for brackets/braces:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/a5b2f620efedf1030fbba7468a3a8556401a8c3a

Deal with structure symbols in string:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/14f8f64f1babef1a07807bb5218e69e2e41cea02

## Test cases
At this point we've hit a milestone success, as all the key combinations of the first simple example I gave have passed. Now I wanted more test cases and maybe even a unit test written for me.

So I asked
```
Great, now can you give me 3 other test cases, ideally it could cover any value types json can have. Let's not have any spaces in the structural parts of the JSON. For each test case, give me
1. a JSON string
2. 3 test key sequences
3. Their corresponding correct output
```

chatGPT then spit out a few good test cases, with good coverage on multiple styles. For example:

```
{"name":"Alice","age":30,"contacts":[{"type":"email","value":"alice@email.com"},{"type":"phone","value":"123-456-7890"}],"isActive":true}
["name"]
["contacts", "type"]
["isActive"]
```

However the expected outputs it gave were completely wrong:
```
[[0, 6], [7, 14]]
[[15, 26], [27, 33], [34, 41], [42, 48], [49, 56], [57, 64]]
[[65, 75], [76, 80]]
```

And no matter how I tried to educate it or have it explain to me why these answers are right or wrong, it could not walk straight.

`At this point I was like back in my TA years, seeing students gone totally blank and fail to explain how they came up with their code that was copied from somewhere else. Except for this time, chatGPT did write the code itself. Or did it?`

## Refinements
I finally gave up educating it, and turned to ask it help me with some refinements and prettification of outputs so I can more easily manually examine whether the output was expected:

https://github.com/chcharcharlie/rust-json-str-redactor/commit/ec4f2dcca526086727a5a2b3c2b45e62d14e3d62

## The last mile
With the pretty printing, I was able to examine a few more test cases and of course found quite a few breaking points. But for the last mile, chatGPT wasn't able to help any further. The code turned too complex and even when I point to it exactly what could have gone wrong, the returned edits would not solve it and brings other breaking changes. I had to do the final debuggings and edits and I `think` I've finally made it work.

https://github.com/chcharcharlie/rust-json-str-redactor/commit/11ca28ac37a9cf6720c5b611cd981a1da14b69ea

## Final thoughts
I'm recording this as I found my experience for the past day pretty intriguring at least to myself. As I first get started and see the hundred line tedious char processing code appear to me within 10 seconds, I was a bit shocked. Later when I found out there are still so many bugs it can't find, or solutions it doesn't understand, I was both a bit annoyed, and a little celebrating inside. At the end, I think I kind of found the rhythm to work together with it, by knowing when I should say something, or when I should take over myself.

At least for today, relying on chatGPT to code is definitely doable, but I would highly recommend that you read and understand the code yourself, and not expect it to magically complete everything for you. But with all the AI advancements this year, who knows what our workflow be like when it's chatGPT 5, 6, 7? Maybe we programmers really will be out of our jobs soon.