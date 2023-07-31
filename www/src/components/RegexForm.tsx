'use client';
import {Field, FieldProps, Form, Formik, FormikHelpers} from 'formik';

export type RegexFormProps = {
  regexHandler: (regex: string) => Promise<void>;
};

type FormValues = {
  regex: string;
};

export default function RegexForm(props: RegexFormProps) {
  const {regexHandler} = props;
  const initialValues: FormValues = {
    regex: '',
  };
  const handleSubmit = async (
    values: FormValues,
    helpers: FormikHelpers<FormValues>
  ) => {
    const {regex} = values;
    await regexHandler(regex);
    helpers.setSubmitting(false);
  };

  return (
    <Formik initialValues={initialValues} onSubmit={handleSubmit}>
      {() => (
        <Form className="bg-white shadow-md rounded px-8 pt-6 pb-8 mb-4">
          <Field name="regex">
            {({field, form}: FieldProps<string, FormValues>) => (
              <div className="mb-4">
                <label
                  className="block text-gray-700 text-sm font-bold mb-2"
                  htmlFor="regex"
                >
                  Regex
                </label>
                <input
                  className="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                  type="text"
                  disabled={form.isSubmitting}
                  {...field}
                />
              </div>
            )}
          </Field>
          <div className="flex items-center justify-between">
            <button
              className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
              type="submit"
            >
              Submit
            </button>
          </div>
        </Form>
      )}
    </Formik>
  );
}
