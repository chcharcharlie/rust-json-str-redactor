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
